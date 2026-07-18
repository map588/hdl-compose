
library IEEE;
	use IEEE.STD_LOGIC_1164.all;
	use IEEE.NUMERIC_STD.all;

entity addressed_reg is
	generic (
		n         : integer := 7;
		addr_size : integer := 7;
		address   : integer
	);
	port (D              : in  STD_LOGIC_VECTOR(n - 1 downto 0);
	      ADDR           : in  STD_LOGIC_VECTOR(addr_size - 1 downto 0);
	      CLK, LOAD, RST : in  STD_LOGIC;
	      Q              : out STD_LOGIC_VECTOR(n - 1 downto 0)
	  );
end entity;

architecture rtl of addressed_reg is

	constant assigned_address : unsigned(addr_size - 1 downto 0) := to_unsigned(address, addr_size);

	component pl_reg is
		generic (n : integer := 7);
		port (D              : in  STD_LOGIC_VECTOR(n - 1 downto 0);
		      CLK, LOAD, RST   : in  STD_LOGIC;
		      Q              : out STD_LOGIC_VECTOR(n - 1 downto 0)
	   );
	end component;

	signal cond_load : std_logic := '0';
	
begin

	u_reg: pl_reg
		generic map (n => n)
		port map (
			D    => D,
			CLK  => CLK,
			LOAD => cond_load,
			RST  => RST,
			Q    => Q
		);

	process (CLK) is
		variable addr_matches : boolean := false;
	begin
		if rising_edge(CLK) then
			addr_matches := unsigned(ADDR) = assigned_address;
			if addr_matches then
				cond_load <= LOAD;
			else
				cond_load <= '0';
			end if;
		end if;
	end process;

end architecture;
