library IEEE;
	use IEEE.STD_LOGIC_1164.all;
	use IEEE.NUMERIC_STD.all;

library loot;
	use loot.midi_lut_pkg.all;

entity phase_incre is
	port (
		clk                 : in  std_logic;
		resetn              : in  std_logic;
		phase_pkg           : in  std_logic_vector(6 downto 0);
		update_freq         : in  std_logic;
		m_axis_phase_tready : in  std_logic;
		m_axis_phase_tvalid : out std_logic;
		m_axis_phase_tdata  : out std_logic_vector(23 downto 0)
	);
end entity;

architecture Behavioral of phase_incre is
	component edge_detector is
		port (
			clk     : in  STD_LOGIC;
			rstn    : in  STD_LOGIC;
			sig_in  : in  STD_LOGIC;
			sig_out : out STD_LOGIC
		);
	end component;

	signal phase_acc     : unsigned(23 downto 0);
	signal phase_inc     : unsigned(23 downto 0);
	signal data_reg      : std_logic_vector(23 downto 0);
	signal valid_reg     : std_logic;
	signal update_latch  : std_logic;
	signal transfer_done : std_logic;
begin
	-- Edge detector for frequency updates
	edge_detect: edge_detector
		port map (
			clk     => clk,
			rstn    => resetn,
			sig_in  => update_freq,
			sig_out => update_latch
		);

	-- Detect successful transfers
	transfer_done <= m_axis_phase_tready and valid_reg;

	process (clk) is
	begin
		if rising_edge(clk) then
			if resetn = '0' then
				phase_acc <= (others => '0');
				phase_inc <= (others => '0');
				valid_reg <= '0';
				data_reg  <= (others => '0');
			else
				if update_latch = '1' then
					-- New frequency requested
					phase_acc <= (others => '0');
					phase_inc <= unsigned(midi_phase_inc_lut(to_integer(unsigned(phase_pkg))));
					valid_reg <= '1';
				elsif transfer_done = '1' then
					-- Transfer completed, generate next phase
					phase_acc <= phase_acc + phase_inc;
					valid_reg <= '1';
				end if;

				-- Update output register when transfer occurs or new value generated
				if transfer_done = '1' or update_latch = '1' then
					data_reg <= std_logic_vector(phase_acc);
				end if;
			end if;
		end if;
	end process;

	-- Output assignments
	m_axis_phase_tdata  <= data_reg;
	m_axis_phase_tvalid <= valid_reg;

end architecture;
