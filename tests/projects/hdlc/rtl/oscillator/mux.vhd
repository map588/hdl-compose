library IEEE;
	use IEEE.STD_LOGIC_1164.all;

entity mux is
	port (
		clk                  : in  std_logic;
		resetn               : in  std_logic; -- Added reset
		sel                  : in  std_logic_vector(1 downto 0);

		-- Saw input
		s_axis_saw_tdata     : in  std_logic_vector(23 downto 0);
		s_axis_saw_tvalid    : in  std_logic;
		s_axis_saw_tready    : out std_logic;

		-- Square input
		s_axis_square_tdata  : in  std_logic_vector(23 downto 0);
		s_axis_square_tvalid : in  std_logic;
		s_axis_square_tready : out std_logic;

		-- Sine input
		s_axis_sine_tdata    : in  std_logic_vector(23 downto 0);
		s_axis_sine_tvalid   : in  std_logic;
		s_axis_sine_tready   : out std_logic;

		-- Output
		m_axis_out_tdata     : out std_logic_vector(23 downto 0);
		m_axis_out_tvalid    : out std_logic;
		m_axis_out_tready    : in  std_logic
	);
end entity;

architecture Behavioral of mux is
	-- Registered outputs
	signal data_reg  : std_logic_vector(23 downto 0);
	signal valid_reg : std_logic;

	-- Ready signals for inactive inputs should be '0'
	signal saw_ready    : std_logic;
	signal square_ready : std_logic;
	signal sine_ready   : std_logic;
begin
	-- Output assignments
	m_axis_out_tdata  <= data_reg;
	m_axis_out_tvalid <= valid_reg;

	-- Ready signals to sources
	s_axis_saw_tready    <= saw_ready;
	s_axis_square_tready <= square_ready;
	s_axis_sine_tready   <= sine_ready;

	process (clk) is
	begin
		if rising_edge(clk) then
			if resetn = '0' then
				data_reg     <= (others => '0');
				valid_reg    <= '0';
				saw_ready    <= '0';
				square_ready <= '0';
				sine_ready   <= '0';
			else
				-- Default all ready signals to '0'
				saw_ready    <= '0';
				square_ready <= '0';
				sine_ready   <= '0';

				case sel is
					when "00"| "11" => -- Saw
						saw_ready <= m_axis_out_tready;
						if s_axis_saw_tvalid = '1' and m_axis_out_tready = '1' then
							data_reg  <= s_axis_saw_tdata;
							valid_reg <= '1';
						elsif m_axis_out_tready = '1' then
							valid_reg <= '0';
						end if;

					when "01" => -- Square
						square_ready <= m_axis_out_tready;
						if s_axis_square_tvalid = '1' and m_axis_out_tready = '1' then
							data_reg  <= s_axis_square_tdata;
							valid_reg <= '1';
						elsif m_axis_out_tready = '1' then
							valid_reg <= '0';
						end if;

					when "10" => -- Sine
						sine_ready <= m_axis_out_tready;
						if s_axis_sine_tvalid = '1' and m_axis_out_tready = '1' then
							data_reg  <= s_axis_sine_tdata;
							valid_reg <= '1';
						elsif m_axis_out_tready = '1' then
							valid_reg <= '0';
						end if;

					when others =>
						data_reg <= (others => '0');
						valid_reg <= '0';
				end case;
			end if;
		end if;
	end process;

end architecture;
