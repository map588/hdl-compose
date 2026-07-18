
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity i2c_cv_dac_tb is
end entity;

architecture sim of i2c_cv_dac_tb is

  constant CLK_PERIOD : time := 10 ns;

  signal clk   : std_logic := '0';
  signal rst_n : std_logic := '0';
  signal ch_a  : std_logic_vector(11 downto 0) := x"ABC";
  signal ch_b  : std_logic_vector(11 downto 0) := x"123";
  signal ch_c  : std_logic_vector(11 downto 0) := x"FFF";
  signal scl   : std_logic;
  signal sda   : std_logic;
  signal ldac  : std_logic;

  signal done : boolean := false;

  type byte_array is array (0 to 8) of std_logic_vector(7 downto 0);
  constant expected : byte_array := (
    x"C0",        -- device address 1100000 + W
    x"0A", x"BC", -- channel A fast write
    x"01", x"23", -- channel B
    x"0F", x"FF", -- channel C
    x"00", x"00"  -- channel D (unused)
  );

begin

  clk <= not clk after CLK_PERIOD / 2 when not done else '0';

  sda <= 'H'; -- bus pull-up

  dut: entity work.i2c_cv_dac
    generic map (CLK_DIV => 4, DEV_ADDR => "1100000")
    port map (
      clk => clk, rst_n => rst_n,
      ch_a => ch_a, ch_b => ch_b, ch_c => ch_c,
      scl => scl, sda => sda, ldac => ldac);

  monitor: process
    variable byte : std_logic_vector(7 downto 0);
    variable ackbit : std_logic;
  begin
    rst_n <= '0';
    wait for 5 * CLK_PERIOD;
    rst_n <= '1';

    assert ldac = '0' report "LDAC not held low" severity error;

    -- wait for START: SDA falls while SCL high
    wait until falling_edge(sda);
    assert to_x01(scl) = '1' report "START: SCL not high at SDA fall" severity error;

    for b in 0 to 8 loop
      for i in 7 downto 0 loop
        wait until rising_edge(scl);
        byte(i) := to_x01(sda);
      end loop;
      assert byte = expected(b)
        report "byte " & integer'image(b) & " wrong"
        severity error;
      -- ACK slot: master must release SDA (pull-up wins -> reads '1')
      wait until rising_edge(scl);
      ackbit := to_x01(sda);
      assert ackbit = '1' report "SDA driven during ACK slot" severity error;
    end loop;

    -- STOP: SDA rises while SCL high
    wait until rising_edge(sda);
    assert to_x01(scl) = '1' report "STOP: SCL not high at SDA rise" severity error;

    -- next frame must reflect new channel values
    ch_a <= x"055";
    wait until falling_edge(sda); -- next START
    for i in 7 downto 0 loop      -- address byte
      wait until rising_edge(scl);
    end loop;
    wait until rising_edge(scl);  -- ack
    byte := (others => '0');
    for i in 7 downto 0 loop      -- ch A high byte
      wait until rising_edge(scl);
      byte(i) := to_x01(sda);
    end loop;
    assert byte = x"00" report "frame 2: ch A high byte wrong" severity error;
    wait until rising_edge(scl);  -- ack
    for i in 7 downto 0 loop      -- ch A low byte
      wait until rising_edge(scl);
      byte(i) := to_x01(sda);
    end loop;
    assert byte = x"55" report "frame 2: ch A low byte not updated" severity error;

    report "i2c_cv_dac_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
