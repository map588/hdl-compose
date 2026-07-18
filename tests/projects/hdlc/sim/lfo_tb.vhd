
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity lfo_tb is
end entity;

architecture sim of lfo_tb is

  constant CLK_PERIOD : time    := 10 ns;
  constant TICK_CLKS  : integer := 10;

  signal clk         : std_logic := '0';
  signal rst_n       : std_logic := '0';
  signal param_write : std_logic := '0';
  signal param_addr  : std_logic_vector(6 downto 0) := (others => '0');
  signal param_data  : std_logic_vector(6 downto 0) := (others => '0');
  signal lfo_out     : std_logic_vector(11 downto 0);

  signal done : boolean := false;

begin

  clk <= not clk after CLK_PERIOD / 2 when not done else '0';

  dut: entity work.lfo
    generic map (RATE_ADDR => 85, CLKS_PER_TICK => TICK_CLKS)
    port map (
      clk => clk, rst_n => rst_n,
      param_write => param_write, param_addr => param_addr, param_data => param_data,
      lfo_out => lfo_out);

  stim: process
    variable prev      : unsigned(11 downto 0);
    variable saw_peak  : boolean := false;
    variable saw_zero  : boolean := false;
  begin
    wait for 5 * CLK_PERIOD;
    rst_n <= '1';

    -- rate = 127 -> step 127/tick -> ~65 ticks up, 65 down
    param_addr <= std_logic_vector(to_unsigned(85, 7));
    param_data <= std_logic_vector(to_unsigned(127, 7));
    wait until rising_edge(clk);
    param_write <= '1';
    wait until rising_edge(clk);
    param_write <= '0';
    for i in 0 to 9 loop
      wait until rising_edge(clk);
    end loop;

    prev := unsigned(lfo_out);
    for t in 1 to 200 * TICK_CLKS loop
      wait until rising_edge(clk);
      if unsigned(lfo_out) = 4095 then
        saw_peak := true;
      end if;
      if saw_peak and unsigned(lfo_out) = 0 then
        saw_zero := true;
      end if;
    end loop;

    assert saw_peak report "LFO never reached peak" severity error;
    assert saw_zero report "LFO never returned to zero after peak" severity error;

    report "lfo_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
