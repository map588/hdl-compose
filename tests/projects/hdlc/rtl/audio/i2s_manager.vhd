library IEEE;
	use IEEE.STD_LOGIC_1164.all;
	use IEEE.NUMERIC_STD.all;

entity i2s_transmitter is
	generic (
		DATA_WIDTH : positive := 24
	);
	port (
		s_axis_in_clk    : in  std_logic; -- System clock (data producer clock)
		s_axis_in_tdata  : in  std_logic_vector(DATA_WIDTH - 1 downto 0);
		s_axis_in_tvalid : in  std_logic; -- s_axis_in_tvalid data into FIFO
		s_axis_in_tready : out std_logic;

		resetn           : in  std_logic; -- Reset (active low)
		bclk             : out std_logic; -- Bit clock (2.304 MHz)
		lrclk            : out std_logic; -- LR clock (96 kHz)
		sdata            : out std_logic  -- Serial data output (DIN for DAC)
	);

	attribute CLOCK_DEDICATED_ROUTE          : string;
	attribute CLOCK_DEDICATED_ROUTE of bclk  : signal is "TRUE";
	attribute CLOCK_DEDICATED_ROUTE of lrclk : signal is "TRUE";

	attribute KEEP          : string;
	attribute KEEP of bclk  : signal is "TRUE";
	attribute KEEP of lrclk : signal is "TRUE";
	attribute KEEP of sdata : signal is "TRUE";

end entity;

architecture Behavioral of i2s_transmitter is
	component i2s is
		generic (
			mclk_sclk_ratio  : INTEGER := 4;  --number of mclk periods per sclk period
			sclk_lrclk_ratio : INTEGER := 64; --number of sclk periods per word select period
			d_width          : INTEGER := 24); --data width
		port (
			resetn  : in  STD_LOGIC;                              --asynchronous active low reset
			mclk    : in  STD_LOGIC;                              --master clock
			sclk    : out STD_LOGIC;                              --serial clock (or bit clock)
			lrclk   : out STD_LOGIC;                              --word select (or left-right clock)
			sdata   : out STD_LOGIC;                              --serial data transmit
			data_tx : in  STD_LOGIC_VECTOR(d_width - 1 downto 0)); --channel data to transmit
	end component;
	signal lrclk_int  : std_logic := '0';
	signal lrclk_prev : std_logic := '0';

begin

	lrclk <= lrclk_int;

	i2s_inst: i2s
		port map (
			mclk    => s_axis_in_clk,
			resetn  => resetn,
			sclk    => bclk,
			lrclk   => lrclk_int,
			sdata   => sdata,
			data_tx => s_axis_in_tdata
		);

	process (s_axis_in_clk, lrclk_int, resetn) is
	begin
		if resetn = '0' then
			s_axis_in_tready <= '0';
			lrclk_prev       <= '0';
		elsif rising_edge(s_axis_in_clk) then
			if lrclk_int /= lrclk_prev then
				s_axis_in_tready <= '1';
			else
				s_axis_in_tready <= '0';
			end if;
			lrclk_prev <= lrclk_int;
		end if;
	end process;

end architecture;
