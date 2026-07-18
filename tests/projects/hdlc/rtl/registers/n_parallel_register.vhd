library IEEE;
  use IEEE.STD_LOGIC_1164.all;

entity pl_reg is
  generic (n : integer := 32);
  port (D              : in  STD_LOGIC_VECTOR(n - 1 downto 0);
        CLK, LOAD, RST : in  STD_LOGIC;
        Q              : out STD_LOGIC_VECTOR(n - 1 downto 0)
          --QN: out  STD_LOGIC_VECTOR(n-1 downto 0)
       );
end entity;

architecture rtl of pl_reg is
  signal reg : STD_LOGIC_VECTOR(n - 1 downto 0);

  component d_ff is
    port (
      CLK : in  std_logic;
      D   : in  std_logic;
      EN  : in  std_logic;
      RST : in  std_logic;
      Q   : out std_logic
    );
  end component;

begin

  gen: for i in 0 to n - 1 generate
  begin
    DFF: d_ff
      port map (
        CLK => CLK,
        D   => D(i),
        EN  => LOAD,
        RST => RST,
        Q   => reg(i)
      );
  end generate;

  Q <= reg;
  --Qn <= not reg;
end architecture;
