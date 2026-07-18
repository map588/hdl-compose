----------------------------------------------------------------------------------
-- Company: 
-- Engineer: 
-- 
-- Create Date: 09/11/2024 08:31:05 PM
-- Design Name: 
-- Module Name: ranger_mapper - Behavioral
-- Project Name: 
-- Target Devices: 
-- Tool Versions: 
-- Description: 
-- 
-- Dependencies: 
-- 
-- Revision:
-- Revision 0.01 - File Created
-- Additional Comments:
-- 
----------------------------------------------------------------------------------

library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity range_mapper is
  generic (
    INPUT_WIDTH  : positive := 8;
    OUTPUT_WIDTH : positive := 2
  );
  port (
    din  : in  STD_LOGIC_VECTOR(INPUT_WIDTH - 1 downto 0);
    dout : out STD_LOGIC_VECTOR(OUTPUT_WIDTH - 1 downto 0)
  );
end entity;


architecture Efficient of range_mapper is

	function diff (A : positive; B : positive) return natural is
		variable pos_diff : natural := 0;
	begin
		if A > B then
			pos_diff := A - B;
		else
			pos_diff := B - A;
		end if;
		return pos_diff;
	end function;


	constant UPSCALE : boolean := OUTPUT_WIDTH > INPUT_WIDTH;
	constant SHIFT_AMOUNT : natural := diff(OUTPUT_WIDTH, INPUT_WIDTH);
begin
	process (din) is
		variable temp : unsigned(OUTPUT_WIDTH - 1 downto 0);
	begin
		if UPSCALE then
			temp	:= resize(unsigned(din), OUTPUT_WIDTH);
			dout  <= std_logic_vector(shift_left(temp, SHIFT_AMOUNT));
		else
			dout  <= din(INPUT_WIDTH - 1 downto SHIFT_AMOUNT);
		end if;
	end process;
end architecture;



