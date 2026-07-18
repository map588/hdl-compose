library ieee;
	use ieee.std_logic_1164.all;
	use ieee.numeric_std.all;

library loot;
	use loot.midi_lut_pkg.all;

entity sawtooth_i is
	generic (
		sample_depth : integer := 24
	);
	port (
		clk               : in  std_logic;
		resetn            : in  std_logic;
		direction		  : in  std_logic;
		m_axis_saw_tready : in  std_logic;
		phase_idx         : in  std_logic_vector(6 downto 0);
		phase_ready       : in  std_logic;
		m_axis_saw_tdata  : out std_logic_vector(sample_depth - 1 downto 0);
		m_axis_saw_tvalid : out std_logic
	);
end entity;

architecture rtl of sawtooth_i is
	constant wrap : integer :=  8388607;   -- 2^23 - 1
	constant init : integer := -8388608; -- -2^23
	type saw_direction is (L , R);

	type saw_intialization is array (saw_direction) of integer;
	
	
	constant saw_init : saw_intialization := (
		L =>  8388607,
		R => -8388608
	);

	constant saw_wrap : saw_intialization := (
		L => -8388608,
		R =>  8388607
	);


	component edge_detector is
		port (
			clk     : in  STD_LOGIC;
			rstn    : in  STD_LOGIC;
			sig_in  : in  STD_LOGIC;
			sig_out : out STD_LOGIC
		);
	end component;

	signal current_direction  : saw_direction := L;
	signal current_dir 	      : std_logic 	  := '0';
	signal prev_dir           : std_logic 	  := '0';	
	signal counter            : integer   	  := saw_init(L);
	signal incr               : integer   	  :=  0;
	signal update_latch       : std_logic 	  := '0';
	signal data_reg           : std_logic_vector(sample_depth - 1 downto 0);
	signal valid_reg          : std_logic 	  := '0';
	signal transfer_complete  : std_logic;
begin
	-- Detect successful transfers
	transfer_complete <= m_axis_saw_tready and valid_reg;

	-- Instantiate the edge detector
	edge_detect: edge_detector
		port map (
			clk     => clk,
			rstn    => resetn,
			sig_in  => phase_ready,
			sig_out => update_latch
		);

	direction_detect: edge_detector
		port map (
			clk     => clk,
			rstn    => resetn,
			sig_in  => direction,
			sig_out => current_dir
		);

	process(clk, current_direction) is
	begin
		if rising_edge(clk) then
			if current_dir = '1' and prev_dir = '0' then
				if current_direction = L then
					current_direction <= R;
				else
					current_direction <= L;
				end if;
			end if;
			prev_dir <= current_dir;
		end if;
	end process;

	-- Main sawtooth generation process
	process (clk, resetn) is
	begin
		if resetn = '0' then
			counter   <= init;
			valid_reg <= '0';
			data_reg  <= (others => '0');
			incr      <= 0;
		elsif rising_edge(clk) then
			if update_latch = '1' then
				-- New phase requested
				counter   <= saw_init(current_direction);
				valid_reg <= '1';
				incr      <= to_integer(unsigned(midi_phase_inc_lut(to_integer(unsigned(phase_idx)))));
			elsif transfer_complete = '1' then
				-- Data was transferred, generate next sample
				case current_direction is
					when R =>
						if counter >= saw_wrap(R) then
							counter <= saw_init(R);
						else
							counter <= counter + incr;
						end if;
					when L =>
						if counter <= saw_wrap(L) then
							counter <= saw_init(L);
						else
							counter <= counter - incr;
						end if;
					end case;
					-- if counter >= wrap then
					-- 	counter <= init;
					-- else
					-- 	counter <= counter + incr;
					-- end if;
				valid_reg <= '1'; -- Keep producing data
			end if;

			-- Update output register when transfer occurs or new value generated
			if transfer_complete = '1' or update_latch = '1' then
				data_reg <= std_logic_vector(to_signed(counter, sample_depth));
			end if;
		end if;
	end process;

	-- Output assignments
	m_axis_saw_tdata  <= data_reg;
	m_axis_saw_tvalid <= valid_reg;

end architecture;
