library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

entity midi_processor is
  generic (
    VOICES : integer := 2
  );
  port (
    clk          : in  std_logic;
    rstn         : in  std_logic;

    update       : in  std_logic;
    data_in      : in  std_logic_vector(15 downto 0);

    triggers     : out std_logic_vector(1 downto 0) := (others => '0');
    note_stops   : out std_logic_vector(1 downto 0) := (others => '0'); 
    note_states  : out std_logic_vector(1 downto 0) := (others => '0');
    note_index   : out std_logic_vector(6 downto 0) := (others => '0');
    velocity     : out std_logic_vector(6 downto 0) := (others => '0');
    param_write  : out std_logic := '0';
    param_number : out std_logic_vector(6 downto 0) := (others => '0');
    param_value  : out std_logic_vector(6 downto 0) := (others => '0');
    read_midi    : out std_logic := '0';
    ack          : out std_logic := '0'
  );
end entity;

architecture Behavioral of midi_processor is
  -- Message type constants
  constant MSG_NOTE_OFF : std_logic_vector(1 downto 0) := "00";
  constant MSG_NOTE_ON  : std_logic_vector(1 downto 0) := "01";
  constant MSG_PARAM    : std_logic_vector(1 downto 0) := "10";

  -- State machine states
  type state_type is (
    IDLE,           -- Waiting for update signal
    CAPTURE,        -- Capture input data and determine message type
    PROCESS_NOTE,   -- Process note on/off messages  
    PROCESS_PARAM,  -- Process parameter messages
    UPDATE_OUTPUTS  -- Update output registers
  );

  signal state, next_state : state_type;

  -- Data registers
  signal data_reg      : std_logic_vector(15 downto 0) := (others => '0');
  signal param_num_reg : std_logic_vector(6 downto 0)  := (others => '0');
  signal param_val_reg : std_logic_vector(6 downto 0)  := (others => '0');
  signal vel_reg       : std_logic_vector(6 downto 0)  := (others => '0');

  -- Voice tracking registers
  type voice_note_array is array (0 to 1) of std_logic_vector(6 downto 0);
  signal voice_notes : voice_note_array := (others => (others => '0'));
  signal voice_age   : std_logic := '0'; -- Tracks which voice is older (alternates)
  
  -- Status registers
  signal triggers_reg    : std_logic_vector(1 downto 0) := (others => '0');
  signal note_stops_reg  : std_logic_vector(1 downto 0) := (others => '0');
  signal note_states_reg : std_logic_vector(1 downto 0) := (others => '0');
  signal param_write_reg : std_logic := '0';

  -- Control signals
  signal msg_type : std_logic_vector(1 downto 0);

begin
  -- State register process
  state_reg: process (clk, rstn)
  begin
    if rstn = '0' then
      state <= IDLE;
    elsif rising_edge(clk) then
      state <= next_state;
    end if;
  end process;

  -- Next state logic
  combinatorial: process (state, update, data_reg)
  begin
    next_state <= state;

    case state is
      when IDLE =>
        if update = '1' then
          next_state <= CAPTURE;
        end if;

      when CAPTURE =>
        msg_type <= data_reg(15 downto 14);
        case data_reg(15 downto 14) is
          when MSG_NOTE_ON | MSG_NOTE_OFF =>
            next_state <= PROCESS_NOTE;
          when MSG_PARAM =>
            next_state <= PROCESS_PARAM;
          when others =>
            next_state <= IDLE;
        end case;

      when PROCESS_NOTE =>
        next_state <= UPDATE_OUTPUTS;

      when PROCESS_PARAM =>
        next_state <= UPDATE_OUTPUTS;

      when UPDATE_OUTPUTS =>
        next_state <= IDLE;
    end case;
  end process;

  -- Registered outputs and data processing
  sequential: process (clk, rstn)
    variable incoming_note : std_logic_vector(6 downto 0);
    variable voice_to_use : integer range 0 to 1;
    variable found_voice : boolean;
  begin
    if rstn = '0' then
      -- Reset all registers
      data_reg <= (others => '0');
      param_num_reg <= (others => '0');
      param_val_reg <= (others => '0');
      vel_reg <= (others => '0');
      voice_notes <= (others => (others => '0'));
      triggers_reg <= (others => '0');
      note_stops_reg <= (others => '0');
      note_states_reg <= (others => '0');
      param_write_reg <= '0';
      voice_age <= '0';

      -- Reset all outputs
      triggers <= (others => '0');
      note_stops <= (others => '0');
      note_states <= (others => '0');
      note_index <= (others => '0');
      velocity <= (others => '0');
      param_write <= '0';
      param_number <= (others => '0');
      param_value <= (others => '0');
      read_midi <= '0';
      ack <= '0';

    elsif rising_edge(clk) then
      -- Default assignments
      triggers_reg <= (others => '0');
      note_stops_reg <= (others => '0');
      param_write <= '0';
      ack <= '0';
      read_midi <= '0';

      case state is
        when IDLE =>
          if update = '1' then
            read_midi <= '1';
            -- latch here so CAPTURE's combinational message-type routing
            -- sees THIS message, not the previous one
            data_reg <= data_in;
          end if;

        when CAPTURE =>
          null;

        when PROCESS_NOTE =>
          incoming_note := data_reg(13 downto 7);
          vel_reg <= data_reg(6 downto 0);
          found_voice := false;

          if msg_type = MSG_NOTE_ON then
            -- First check if note is already playing
            for i in 0 to 1 loop
              if voice_notes(i) = incoming_note then
                -- Retrigger existing note
                voice_to_use := i;
                found_voice := true;
                exit;
              end if;
            end loop;

            if not found_voice then
              -- Find empty voice or replace oldest
              if voice_notes(0) = "0000000" then
                voice_to_use := 0;
              elsif voice_notes(1) = "0000000" then
                voice_to_use := 1;
              else
                -- Replace oldest voice
                if voice_age = '0' then
                  voice_to_use := 0;
                else
                  voice_to_use := 1;
                end if;
                voice_age <= not voice_age;
              end if;
            end if;

            -- Activate new note
            voice_notes(voice_to_use) <= incoming_note;
            triggers_reg(voice_to_use) <= '1';
            note_states_reg(voice_to_use) <= '1';
            note_index <= incoming_note;

          else -- MSG_NOTE_OFF
            -- Find and deactivate matching note
            for i in 0 to 1 loop
              if voice_notes(i) = incoming_note then
                voice_notes(i) <= (others => '0');
                note_stops_reg(i) <= '1';
                note_states_reg(i) <= '0';
                exit;
              end if;
            end loop;
          end if;

        when PROCESS_PARAM =>
          param_num_reg <= data_reg(13 downto 7);
          param_val_reg <= data_reg(6 downto 0);
          param_write_reg <= '1';

        when UPDATE_OUTPUTS =>
          if msg_type = MSG_NOTE_ON or msg_type = MSG_NOTE_OFF then
            triggers <= triggers_reg;
            note_stops <= note_stops_reg;
            note_states <= note_states_reg;
            velocity <= vel_reg;
          elsif msg_type = MSG_PARAM then
            param_write <= param_write_reg;
            param_number <= param_num_reg;
            param_value <= param_val_reg;
          end if;
          ack <= '1';
      end case;
    end if;
  end process;

end architecture;