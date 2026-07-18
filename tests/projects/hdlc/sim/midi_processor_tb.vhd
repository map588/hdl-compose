library IEEE;
  use IEEE.std_logic_1164.all;
  use IEEE.numeric_std.all;

library work;
  use work.midi_tb_pkg.all;

entity midi_processor_tb is
end entity;

architecture behavior of midi_processor_tb is
  constant VOICES : integer := 2;
  -- Component declaration
  component midi_interface is
    port (
      sys_clk     : in  std_logic;
      sys_rstn    : in  std_logic;
      midi_clk    : in  std_logic;
      midi_sdata  : in  std_logic;
      midi_startn : in  std_logic;
      cpu_ack     : in  std_logic := '0';
      midi_output : out std_logic_vector(15 downto 0);
      interrupt   : out std_logic;       -- Towards CPU
      update      : out std_logic := '0' -- Towards PL
    );
  end component;

  component midi_processor is
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
      note_index   : out std_logic_vector(6 downto 0)          := (others => '0');
      velocity     : out std_logic_vector(6 downto 0)          := (others => '0');
      param_write  : out std_logic                             := '0';
      param_number : out std_logic_vector(6 downto 0)          := (others => '0');
      param_value  : out std_logic_vector(6 downto 0)          := (others => '0');
      read_midi    : out std_logic                             := '0';
      ack          : out std_logic                             := '0'
    );
  end component;

  -- Stimulus signals
  signal sys_clk     : std_logic := '0';
  signal sys_rstn    : std_logic := '0';
  signal midi_clk    : std_logic := '0';
  signal midi_sdata  : std_logic := '0';
  signal midi_start : std_logic  := '0';

  signal triggers     : std_logic_vector(VOICES - 1 downto 0) := (others => '0');
  signal note_states  : std_logic_vector(VOICES - 1 downto 0) := (others => '0');
  signal note_index   : std_logic_vector(6 downto 0) := (others => '0');
  signal velocity     : std_logic_vector(6 downto 0) := (others => '0');
  signal param_write  : std_logic                    := '0';
  signal param_number : std_logic_vector(6 downto 0) := (others => '0');
  signal param_value  : std_logic_vector(6 downto 0) := (others => '0');
  signal ack          : std_logic                    := '0';
  signal read_midi    : std_logic                    := '0';
  signal update      : std_logic                    := '0';

  -- Output signals
  signal midi_output : std_logic_vector(15 downto 0);
  signal interrupt   : std_logic;

  -- Test procedure
  procedure test_midi_message(
             msg_type   :     midi_message_type;
             index      :     natural range 0 to 127;
             value      :     natural range 0 to 127;
      signal midi_clk   : out std_logic;
      signal midi_data  : out std_logic;
      signal midi_start : out std_logic
    ) is
    variable msg : std_logic_vector(15 downto 0);
  begin
    msg := create_midi_message(msg_type, index, value);
    send_midi_message(midi_clk, midi_data, midi_start, msg);
  end procedure;

begin

  -- Instantiate DUT
  MIDI_IN: midi_interface
    port map (
      sys_clk     => sys_clk,
      sys_rstn    => sys_rstn,
      midi_clk    => midi_clk,
      midi_sdata  => midi_sdata,
      midi_startn  => midi_start,
      cpu_ack     => ack,
      midi_output => midi_output,
      interrupt   => interrupt,
      update      => update
    );

    DUT: midi_processor
    port map(
        clk          => sys_clk,
        rstn         => sys_rstn,
        update       => update,
        data_in      => midi_output,
        triggers     => triggers,
        note_states  => note_states,
        note_index   => note_index,
        velocity     => velocity,
        param_write  => param_write,
        param_number => param_number,
        param_value  => param_value,
        read_midi => read_midi,
        ack          => ack
    );

  -- Clock process

  sys_clk_process: process
  begin
    sys_clk <= '0';
    wait for CLK_PERIOD / 2;
    sys_clk <= '1';
    wait for CLK_PERIOD / 2;
  end process;

  -- Stimulus process

  stim_proc: process
  begin
    -- Reset
    sys_rstn <= '0';
    wait for CLK_PERIOD * 10;
    sys_rstn <= '1';
    wait for CLK_PERIOD * 2;

    -- Test Case 1: ON1 - OFF1
    test_midi_message(NOTE_ON, 60, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_OFF, 60, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    -- Test Case 2: ON1 - ON2 - OFF1 - OFF2
    test_midi_message(NOTE_ON, 70, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_ON, 65, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_OFF, 70, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_OFF, 65, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    -- Test Case 3: ON1 - ON2 - OFF2 - OFF1 
    test_midi_message(NOTE_ON, 70, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_ON, 65, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_OFF, 65, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    test_midi_message(NOTE_OFF, 70, 100, midi_clk, midi_sdata, midi_start);
    wait until ack = '1';
    wait for MIDI_CLK_PERIOD;

    -- End simulation
    wait;
  end process;

end architecture;
