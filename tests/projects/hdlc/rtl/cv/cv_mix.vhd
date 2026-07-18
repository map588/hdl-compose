
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- VCF control voltage: envelope plus depth-scaled bipolar LFO, saturating.
-- cv = clamp( env + ((lfo - 2048) * depth) / 128 )

entity cv_mix is
  generic (
    DEPTH_ADDR : integer range 0 to 127 := 86
  );
  port (
    clk         : in  std_logic;
    rst_n       : in  std_logic;
    param_write : in  std_logic;
    param_addr  : in  std_logic_vector(6 downto 0);
    param_data  : in  std_logic_vector(6 downto 0);
    env_in      : in  std_logic_vector(11 downto 0);
    lfo_in      : in  std_logic_vector(11 downto 0);
    cv_out      : out std_logic_vector(11 downto 0)
  );
end entity;

architecture Behavioral of cv_mix is

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

  signal depth : std_logic_vector(6 downto 0);

begin

  depth_reg: map_reg
    generic map (n => 7, addr_size => 7, address => DEPTH_ADDR, output_width => 7)
    port map (D => param_data, ADDR => param_addr, CLK => clk, LOAD => param_write, RSTN => rst_n, Q => depth);

  mix: process (clk)
    variable lfo_c : signed(13 downto 0);
    variable prod  : signed(21 downto 0);
    variable sum   : signed(14 downto 0);
  begin
    if rising_edge(clk) then
      if rst_n = '0' then
        cv_out <= (others => '0');
      else
        lfo_c := signed(resize(unsigned(lfo_in), 14)) - to_signed(2048, 14);
        prod  := lfo_c * signed('0' & depth);
        sum   := resize(signed(resize(unsigned(env_in), 14)), 15) + resize(shift_right(prod, 7), 15);
        if sum < 0 then
          cv_out <= (others => '0');
        elsif sum > to_signed(4095, 15) then
          cv_out <= (others => '1');
        else
          cv_out <= std_logic_vector(resize(unsigned(sum(11 downto 0)), 12));
        end if;
      end if;
    end if;
  end process;

end architecture;
