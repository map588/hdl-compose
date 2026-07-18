
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity enveloper_tb is
end entity;

architecture sim of enveloper_tb is

  constant CLK_PERIOD : time    := 10 ns;
  constant TICK_CLKS  : integer := 20;

  signal clk         : std_logic := '0';
  signal rst_n       : std_logic := '0';
  signal gate        : std_logic := '0';
  signal trig        : std_logic := '0';
  signal param_write : std_logic := '0';
  signal param_addr  : std_logic_vector(6 downto 0) := (others => '0');
  signal param_data  : std_logic_vector(6 downto 0) := (others => '0');
  signal gate_out    : std_logic;
  signal env_out     : std_logic_vector(11 downto 0);

  signal done : boolean := false;

begin

  clk <= not clk after CLK_PERIOD / 2 when not done else '0';

  dut: entity work.enveloper
    generic map (
      ATTACK_ADDR => 73, DECAY_ADDR => 75, SUSTAIN_ADDR => 70, RELEASE_ADDR => 72,
      CLKS_PER_TICK => TICK_CLKS)
    port map (
      clk => clk, rst_n => rst_n, gate => gate, trig => trig,
      param_write => param_write, param_addr => param_addr, param_data => param_data,
      gate_out => gate_out, env_out => env_out);

  stim: process
    procedure write_param(addr : integer; value : integer) is
    begin
      param_addr <= std_logic_vector(to_unsigned(addr, 7));
      param_data <= std_logic_vector(to_unsigned(value, 7));
      wait until rising_edge(clk);
      param_write <= '1';
      wait until rising_edge(clk);
      param_write <= '0';
      -- map_reg load path is several registered stages; hold the address
      for i in 0 to 9 loop
        wait until rising_edge(clk);
      end loop;
    end procedure;

    procedure wait_ticks(n : integer) is
    begin
      for i in 1 to n * TICK_CLKS loop
        wait until rising_edge(clk);
      end loop;
    end procedure;

    variable peak : unsigned(11 downto 0);
  begin
    wait for 5 * CLK_PERIOD;
    rst_n <= '1';
    wait for 5 * CLK_PERIOD;

    -- CC 100 -> inverted 27 -> step 27*32 = 864/tick (fast A/D/R)
    write_param(73, 100); -- attack
    write_param(75, 100); -- decay
    write_param(70, 64);  -- sustain level = 64*32 = 2048
    write_param(72, 100); -- release

    assert env_out = x"000" report "env not zero at idle" severity error;

    -- full cycle: attack to max, decay to sustain
    gate <= '1';
    wait_ticks(8); -- 4095/864 = 5 ticks to peak
    peak := unsigned(env_out);
    assert peak = 4095 or peak >= 2048
      report "attack did not raise envelope (env=" & integer'image(to_integer(peak)) & ")"
      severity error;
    wait_ticks(8); -- decay finished
    assert unsigned(env_out) = 2048
      report "sustain level wrong (env=" & integer'image(to_integer(unsigned(env_out))) & ", expected 2048)"
      severity error;
    assert gate_out = '1' report "gate_out low while note held" severity error;

    -- release to zero
    gate <= '0';
    wait_ticks(8);
    assert unsigned(env_out) = 0
      report "release did not reach zero (env=" & integer'image(to_integer(unsigned(env_out))) & ")"
      severity error;
    assert gate_out = '0' report "gate_out high after release" severity error;

    -- short gate: release from mid-attack
    gate <= '1';
    wait_ticks(2);
    gate <= '0';
    assert unsigned(env_out) > 0 report "no envelope during short note" severity error;
    wait_ticks(8);
    assert unsigned(env_out) = 0 report "short-note release did not reach zero" severity error;

    -- retrigger: from sustain, trig pulse restarts attack
    gate <= '1';
    wait_ticks(16); -- settle at sustain
    assert unsigned(env_out) = 2048 report "did not settle at sustain before retrigger" severity error;
    trig <= '1';
    wait until rising_edge(clk);
    wait until rising_edge(clk);
    trig <= '0';
    wait_ticks(3);
    assert unsigned(env_out) > 2048
      report "retrigger did not restart attack (env=" & integer'image(to_integer(unsigned(env_out))) & ")"
      severity error;
    gate <= '0';
    wait_ticks(10);

    report "enveloper_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
