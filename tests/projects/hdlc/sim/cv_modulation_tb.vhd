
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- Smoke test for the whole CV section: note-on gates the envelopes and the
-- I2C stream must carry a rising channel-A value.

entity cv_modulation_tb is
end entity;

architecture sim of cv_modulation_tb is

  constant CLK_PERIOD : time    := 10 ns;
  constant MS_CLKS    : integer := 100; -- shrunken "1 ms" tick for sim

  signal clk          : std_logic := '0';
  signal rst_n        : std_logic := '0';
  signal note_states  : std_logic_vector(1 downto 0) := "00";
  signal triggers     : std_logic_vector(1 downto 0) := "00";
  signal param_write  : std_logic := '0';
  signal param_number : std_logic_vector(6 downto 0) := (others => '0');
  signal param_value  : std_logic_vector(6 downto 0) := (others => '0');
  signal env_gates    : std_logic_vector(1 downto 0);
  signal scl          : std_logic;
  signal sda          : std_logic;
  signal ldac         : std_logic;

  signal done : boolean := false;

begin

  clk <= not clk after CLK_PERIOD / 2 when not done else '0';
  sda <= 'H';

  dut: entity work.cv_modulation
    generic map (CLKS_PER_MS => MS_CLKS)
    port map (
      clk => clk, rst_n => rst_n,
      note_states => note_states, triggers => triggers,
      param_write => param_write, param_number => param_number, param_value => param_value,
      env_gates => env_gates, scl => scl, sda => sda, ldac => ldac);

  stim: process
    procedure write_param(addr : integer; value : integer) is
    begin
      param_number <= std_logic_vector(to_unsigned(addr, 7));
      param_value  <= std_logic_vector(to_unsigned(value, 7));
      wait until rising_edge(clk);
      param_write <= '1';
      wait until rising_edge(clk);
      param_write <= '0';
      for i in 0 to 9 loop
        wait until rising_edge(clk);
      end loop;
    end procedure;

    variable ch_a_val : std_logic_vector(15 downto 0);
    variable byte     : std_logic_vector(7 downto 0);
    variable first_a  : unsigned(11 downto 0);
    variable later_a  : unsigned(11 downto 0) := (others => '0');
    variable got_rise : boolean := false;
  begin
    wait for 5 * CLK_PERIOD;
    rst_n <= '1';
    wait for 5 * CLK_PERIOD;

    write_param(73, 100); -- VCA attack (fast)
    write_param(75, 100);
    write_param(70, 64);
    write_param(72, 100);
    write_param(85, 127); -- LFO rate max
    write_param(86, 64);  -- LFO depth

    -- note on, voice 0
    triggers    <= "01";
    note_states <= "01";
    wait until rising_edge(clk);
    wait until rising_edge(clk);
    triggers <= "00";

    -- watch several I2C frames; channel A must rise as the envelope attacks
    for frame in 1 to 12 loop
      wait until falling_edge(sda) and to_x01(scl) = '1'; -- START
      for i in 7 downto 0 loop                            -- address byte
        wait until rising_edge(scl);
      end loop;
      wait until rising_edge(scl);                        -- ack
      ch_a_val := (others => '0');
      for i in 15 downto 8 loop
        wait until rising_edge(scl);
        ch_a_val(i) := to_x01(sda);
      end loop;
      wait until rising_edge(scl);                        -- ack
      for i in 7 downto 0 loop
        wait until rising_edge(scl);
        ch_a_val(i) := to_x01(sda);
      end loop;
      if frame = 1 then
        first_a := unsigned(ch_a_val(11 downto 0));
      else
        later_a := unsigned(ch_a_val(11 downto 0));
        if later_a > first_a then
          got_rise := true;
        end if;
      end if;
      -- skip to end of frame: 6 more bytes + acks, then STOP
      for b in 1 to 6 loop
        wait until rising_edge(scl); -- ack of previous byte
        for i in 7 downto 0 loop
          wait until rising_edge(scl);
        end loop;
      end loop;
      wait until rising_edge(scl); -- final ack
    end loop;

    assert got_rise
      report "channel A never rose after note-on (first=" &
             integer'image(to_integer(first_a)) & " later=" &
             integer'image(to_integer(later_a)) & ")"
      severity error;
    assert env_gates(0) = '1' report "env gate 0 not active" severity error;

    report "cv_modulation_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
