
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- Free-running triangle LFO. Rate comes from the MIDI param bus.
-- With a 100 us tick and a 7-bit step: rate CC 1 -> ~1.2 Hz, CC 127 -> ~155 Hz.

entity lfo is
  generic (
    RATE_ADDR     : integer range 0 to 127 := 85;
    CLKS_PER_TICK : integer               := 10417 -- ~100 us tick
  );
  port (
    clk         : in  std_logic;
    rst_n       : in  std_logic;
    param_write : in  std_logic;
    param_addr  : in  std_logic_vector(6 downto 0);
    param_data  : in  std_logic_vector(6 downto 0);
    lfo_out     : out std_logic_vector(11 downto 0)
  );
end entity;

architecture Behavioral of lfo is

  component map_reg is
    generic (
      n            : integer  := 7;
      addr_size    : integer  := 7;
      address      : integer;
      output_width : positive := 12
    );
    port (
      D    : in  STD_LOGIC_VECTOR(n - 1 downto 0);
      ADDR : in  STD_LOGIC_VECTOR(addr_size - 1 downto 0);
      CLK  : in  STD_LOGIC;
      LOAD : in  STD_LOGIC;
      RSTN : in  STD_LOGIC;
      Q    : out STD_LOGIC_VECTOR(output_width - 1 downto 0)
    );
  end component;

  signal rate      : std_logic_vector(6 downto 0);
  signal value     : unsigned(11 downto 0) := (others => '0');
  signal rising    : std_logic := '1';
  signal tick      : std_logic := '0';
  signal clk_count : integer range 0 to CLKS_PER_TICK - 1 := 0;

begin

  rate_reg: map_reg
    generic map (n => 7, addr_size => 7, address => RATE_ADDR, output_width => 7)
    port map (D => param_data, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => rate);

  tick_gen: process (clk)
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        clk_count <= 0;
        tick      <= '0';
      elsif clk_count = CLKS_PER_TICK - 1 then
        clk_count <= 0;
        tick      <= '1';
      else
        clk_count <= clk_count + 1;
        tick      <= '0';
      end if;
    end if;
  end process;

  triangle: process (clk)
    variable step : unsigned(11 downto 0);
    variable sum  : unsigned(12 downto 0);
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        value  <= (others => '0');
        rising <= '1';
      elsif tick = '1' then
        if unsigned(rate) = 0 then
          step := to_unsigned(1, 12);
        else
          step := resize(unsigned(rate), 12);
        end if;

        if rising = '1' then
          sum := ('0' & value) + step;
          if sum >= to_unsigned(4095, 13) then
            value  <= (others => '1');
            rising <= '0';
          else
            value <= sum(11 downto 0);
          end if;
        else
          if value <= step then
            value  <= (others => '0');
            rising <= '1';
          else
            value <= value - step;
          end if;
        end if;
      end if;
    end if;
  end process;

  lfo_out <= std_logic_vector(value);

end architecture;
