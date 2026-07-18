library ieee;
	use ieee.std_logic_1164.all;
	use ieee.numeric_std.all;
	use ieee.math_real.all;

entity square is
	port (
		clk                  : in  std_logic; -- Renamed for clarity
		resetn               : in  std_logic;
		s_axis_square_tvalid : in  std_logic;
		s_axis_square_tdata  : in  std_logic_vector(23 downto 0);
		s_axis_square_tready : out std_logic;
		m_axis_square_tdata  : out std_logic_vector(23 downto 0);
		m_axis_square_tvalid : out std_logic;
		m_axis_square_tready : in  std_logic
	);
end entity;

architecture rtl of square is
	type level_type is (pos, neg, off);
	type transition_sequence is array (0 to 3) of std_logic_vector(23 downto 0);

	constant pos_output : signed(23 downto 0) := X"7FFFFF";
	constant neg_output : signed(23 downto 0) := X"800000";
	constant offset     : signed(23 downto 0) := X"1FFFFF";

	constant offset_table : transition_sequence := (
		std_logic_vector(offset),
		std_logic_vector(offset + offset),
		std_logic_vector(offset + offset + offset),
		std_logic_vector(offset + offset + offset + offset)
	);

	-- Pre-calculate transition sequences
	constant neg_transition : transition_sequence := (
		std_logic_vector(pos_output - signed(offset_table(0))),
		std_logic_vector(pos_output - signed(offset_table(1))),
		std_logic_vector(pos_output - signed(offset_table(2))),
		std_logic_vector(pos_output - signed(offset_table(3)))
	);

	constant pos_transition : transition_sequence := (
		std_logic_vector(neg_output + signed(offset_table(0))),
		std_logic_vector(neg_output + signed(offset_table(1))),
		std_logic_vector(neg_output + signed(offset_table(2))),
		std_logic_vector(neg_output + signed(offset_table(3)))
	);

	-- State and control signals
	signal level_state      : level_type           := off;
	signal prev_level_state : level_type           := off;
	signal transition_index : integer range 0 to 4 := 0;
	signal data_reg         : std_logic_vector(23 downto 0);
	signal valid_reg        : std_logic;
	signal transfer_done    : std_logic;

	-- Phase threshold for comparison (MSB indicates sign)
	constant PHASE_THRESHOLD : unsigned(23 downto 0) := X"800000";
begin
	-- AXI-Stream transfer detection
	transfer_done <= m_axis_square_tready and valid_reg;

	-- Input ready when we can process new data
	s_axis_square_tready <= m_axis_square_tready or not valid_reg;

	process (clk) is
	begin
		if rising_edge(clk) then
			if resetn = '0' then
				level_state      <= off;
				prev_level_state <= off;
				transition_index <= 0;
				valid_reg        <= '0';
				data_reg         <= (others => '0');
			else
				-- Capture previous state for transition detection
				prev_level_state <= level_state;

				-- Process input phase when valid
				if s_axis_square_tvalid = '1' then
					-- Compare phase accumulator value to determine square wave state
					if unsigned(s_axis_square_tdata) < PHASE_THRESHOLD then
						level_state <= pos;
					else
						level_state <= neg;
					end if;
					valid_reg <= '1';
				end if;

				-- Handle transitions and output generation
				if transfer_done = '1' then
					-- State transition detection
					if level_state /= prev_level_state then
						transition_index <= 0;
					elsif transition_index < 4 then
						transition_index <= transition_index + 1;
					end if;

					-- Generate output based on state
					case level_state is
						when pos =>
							if transition_index < 4 then
								data_reg <= pos_transition(transition_index);
							else
								data_reg <= std_logic_vector(pos_output);
							end if;
						when neg =>
							if transition_index < 4 then
								data_reg <= neg_transition(transition_index);
							else
								data_reg <= std_logic_vector(neg_output);
							end if;
						when others =>
							data_reg <= (others => '0');
					end case;
				end if;
			end if;
		end if;
	end process;

	-- Output assignments
	m_axis_square_tdata  <= data_reg;
	m_axis_square_tvalid <= valid_reg;

end architecture;
