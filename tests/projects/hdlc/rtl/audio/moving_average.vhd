library ieee;
	use ieee.std_logic_1164.all;
	use ieee.numeric_std.all;

entity moving_average is
	port (
		aclk          : in  std_logic;
		aresetn       : in  std_logic;

		s_axis_tdata  : in  std_logic_vector(23 downto 0);
		s_axis_tvalid : in  std_logic;
		s_axis_tready : out std_logic;

		m_axis_tdata  : out std_logic_vector(23 downto 0);
		m_axis_tvalid : out std_logic;
		m_axis_tready : in  std_logic
	);

end entity;

architecture rtl of moving_average is
	type sample_array_t is array (0 to 3) of signed(23 downto 0);
	signal samples       : sample_array_t;
	signal valid_delay   : std_logic_vector(3 downto 0);
	signal data_reg      : std_logic_vector(23 downto 0);
	signal valid_reg     : std_logic;
	signal transfer_done : std_logic;
	signal input_ready   : std_logic;
	signal sum		     : signed(25 downto 0);

	attribute USE_DSP : string;
	attribute USE_DSP of sum : signal is "yes";
	attribute USE_DSP of samples : signal is "yes";
begin
	-- Transfer detection
	transfer_done <= m_axis_tready and valid_reg;

	-- Input ready when output is ready or output isn't valid
	input_ready   <= m_axis_tready or not valid_reg;
	s_axis_tready <= input_ready;

	process (aclk)
	begin
		if rising_edge(aclk) then
			if aresetn = '0' then
				samples     <= (others => (others => '0'));
				valid_delay <= (others => '0');
				data_reg    <= (others => '0');
				valid_reg   <= '0';
			else
				-- Only process new data when we can accept it
				if s_axis_tvalid = '1' and input_ready = '1' then
					-- Shift samples
					samples <= signed(s_axis_tdata) & samples(0 to 2);

					-- Propagate valid through pipeline
					valid_delay <= valid_delay(2 downto 0) & '1';

					-- Calculate sum and divide by 4 (right shift by 2)
					sum <= resize(samples(0), 26) + resize(samples(1), 26) + resize(samples(2), 26) + resize(signed(s_axis_tdata), 26);

					-- Update output registers
					data_reg <= std_logic_vector(sum(25 downto 2));
					if valid_delay(2) = '1' then -- Buffer is full
						valid_reg <= '1';
					end if;
				elsif transfer_done = '1' then
					-- Clear valid after successful transfer if no new data
					if s_axis_tvalid = '0' then
						valid_reg <= '0';
					end if;
				end if;
			end if;
		end if;
	end process;

	-- Output assignments
	m_axis_tdata  <= data_reg;
	m_axis_tvalid <= valid_reg;

end architecture;
