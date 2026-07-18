library IEEE;
  use IEEE.STD_LOGIC_1164.all;

entity clock_obuf is
  port (
    clk_to_pins : out std_logic; -- Clock output to pins
    clk_in      : in  std_logic; -- Fast clock input from PLL/MMCM
    clk_reset   : in  std_logic  -- Reset signal
  );
end entity;

architecture Behavioral of clock_obuf is
  signal clock_enable   : std_logic := '1';
  signal clk_fwd_out    : std_logic;
  signal clk_in_int_buf : std_logic;
begin
  -- Direct assignment for internal clock
  clk_in_int_buf <= clk_in;

  -- Clock forwarding (was ODDR SAME_EDGE with D1='1'/D2='0', i.e. the output
  -- tracks the clock). A dual-edge process is not synthesizable outside the
  -- vendor primitive, and its net effect here is the clock itself.
  clk_fwd_out <= '0' when clk_reset = '1'
                 else clk_in_int_buf when clock_enable = '1'
                 else '0';

  -- Output buffer (was OBUF): plain signal assignment in vendor-neutral VHDL.
  clk_to_pins <= clk_fwd_out;

end architecture;
