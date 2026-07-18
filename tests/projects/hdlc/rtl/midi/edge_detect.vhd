library IEEE;
	use IEEE.STD_LOGIC_1164.all;

entity edge_detector is
	port (
		clk      : in  STD_LOGIC;
		rstn     : in  STD_LOGIC;
		sig_in   : in  STD_LOGIC;
		sig_out  : out STD_LOGIC
	);
end entity;

architecture Behavioral of edge_detector is
	signal was_high   : STD_LOGIC := '0';
	signal still_high : STD_LOGIC := '0';
begin
	-- Just latch whether it was high
    process (clk, rstn, sig_in) is
    begin
        if rstn = '0' then
            was_high <= '0';
        elsif falling_edge(clk) then
        was_high   <= sig_in;
        still_high <= was_high;
        end if;
    end process;

	-- Direct combinational output
	sig_out <= '0' when still_high = '1' else sig_in;

end architecture;
