
-- SIMULATION-ONLY VHDL twin of the Verilog `selectio` module
-- (clocked_data_out.v) so GHDL can bind it. Vendor-neutral behavioral model;
-- never add this file to the Vivado project - Vivado uses the Verilog module.

library IEEE;
  use IEEE.STD_LOGIC_1164.all;

entity selectio is
  port (
    data_out_from_device : in  std_logic;
    data_out_to_pins     : out std_logic;
    clk_to_pins          : out std_logic;
    clk_in               : in  std_logic;
    clk_reset            : in  std_logic;
    io_reset             : in  std_logic
  );
end entity;

architecture sim of selectio is
  signal clk_fwd_out : std_logic;
begin

  -- Data output buffer (was OBUF): plain signal assignment.
  data_out_to_pins <= data_out_from_device;

  -- Clock forwarding (was ODDR SAME_EDGE with D1='1'/D2='0', i.e. the output
  -- tracks the clock). A dual-edge process is not synthesizable outside the
  -- vendor primitive, and its net effect here is the clock itself.
  clk_fwd_out <= '0' when clk_reset = '1' else clk_in;

  -- Clock output buffer (was OBUF): plain signal assignment.
  clk_to_pins <= clk_fwd_out;

end architecture;
