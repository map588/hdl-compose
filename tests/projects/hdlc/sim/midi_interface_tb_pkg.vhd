library IEEE;
  use IEEE.std_logic_1164.all;
  use IEEE.numeric_std.all;

package midi_tb_pkg is

  -- Clock period definitions (100MHz system clock)
  constant CLK_PERIOD : time := 10 ns;

  -- MIDI clock timing (500kHz)
  constant MIDI_CLK_PERIOD : time := 2000 ns; -- 500kHz

  -- Types for test vectors
  type midi_message_type is (NOTE_OFF, NOTE_ON, PARAM);
  type midi_message_record is record
    msg_type : midi_message_type;
    index    : natural range 0 to 127;
    value    : natural range 0 to 127;
  end record;

  -- Helper function to create 16-bit MIDI message
  function create_midi_message(
    msg_type : midi_message_type;
    index    : natural range 0 to 127;
    value    : natural range 0 to 127
  ) return std_logic_vector;

  -- Procedure to send one MIDI message
  procedure send_midi_message(
    signal midi_clk   : out std_logic;
    signal midi_data  : out std_logic;
    signal midi_start : out std_logic;
           msg        : in  std_logic_vector(15 downto 0)
  );

end package;

package body midi_tb_pkg is

  -- Function to create 16-bit MIDI message
  function create_midi_message(
      msg_type : midi_message_type;
      index    : natural range 0 to 127;
      value    : natural range 0 to 127
    ) return std_logic_vector is
    variable msg : std_logic_vector(15 downto 0);
  begin
    case msg_type is
      when NOTE_OFF => msg(15 downto 14) := "00";
      when NOTE_ON => msg(15 downto 14) := "01";
      when PARAM => msg(15 downto 14) := "10";
    end case;

    msg(13 downto 7) := std_logic_vector(to_unsigned(index, 7));
    msg(6 downto 0) := std_logic_vector(to_unsigned(value, 7));

    return msg;
  end function;

  -- Procedure to send one MIDI message  
  procedure send_midi_message(
      signal midi_clk   : out std_logic;
      signal midi_data  : out std_logic;
      signal midi_start : out std_logic;
             msg        : in  std_logic_vector(15 downto 0)
    ) is
  begin
    -- Initial state
    midi_clk <= '0';
    midi_data <= '0';
    midi_start <= '0';
    wait for MIDI_CLK_PERIOD / 2;

    -- Start frame
    midi_start <= '1';
    wait for MIDI_CLK_PERIOD / 2;

    -- Send 16 bits MSB first
    for i in 15 downto 0 loop
      midi_clk <= '1';
      midi_data <= msg(i);
      wait for MIDI_CLK_PERIOD / 2;
      midi_clk <= '0';
      wait for MIDI_CLK_PERIOD / 2;
    end loop;

    -- End frame
    midi_start <= '0';
    wait for MIDI_CLK_PERIOD;
  end procedure;

end package body;
