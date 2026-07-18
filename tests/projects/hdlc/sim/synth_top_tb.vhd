
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

library work;
  use work.midi_tb_pkg.all;

-- End-to-end smoke test: MIDI serial in -> midi_interface -> fifo ->
-- midi_processor -> envelopes -> I2C DAC stream out.
-- Audio pin outputs are not checked here.

entity synth_top_tb is
end entity;

architecture sim of synth_top_tb is

  signal clk         : std_logic := '0';
  signal mclk        : std_logic := '0';
  signal rstn        : std_logic := '0';
  signal io_reset    : std_logic;
  signal midi_clk    : std_logic := '0';
  signal midi_data   : std_logic := '0';
  signal midi_startn : std_logic := '0';

  signal sdata0, bclk_out0, lrclk_out0, mclk_out0 : std_logic;
  signal sdata1, bclk_out1, lrclk_out1, mclk_out1 : std_logic;
  signal env_gate0, env_gate1 : std_logic;
  signal dac_sclk, dac_ss : std_logic;
  signal dac_mosi : std_logic;

  signal done : boolean := false;

begin

  clk  <= not clk after 5 ns when not done else '0';
  mclk <= not mclk after 40 ns when not done else '0';
  io_reset <= not rstn;
  dac_mosi <= 'H'; -- I2C pull-up

  dut: entity work.synth_top
    port map (
      clk => clk, rstn => rstn,
      midi_clk => midi_clk, midi_data => midi_data, midi_startn => midi_startn,
      mclk => mclk, io_reset => io_reset,
      sdata0 => sdata0, bclk_out0 => bclk_out0, lrclk_out0 => lrclk_out0, mclk_out0 => mclk_out0,
      sdata1 => sdata1, bclk_out1 => bclk_out1, lrclk_out1 => lrclk_out1, mclk_out1 => mclk_out1,
      env_gate0 => env_gate0, env_gate1 => env_gate1,
      DAC_SCLK => dac_sclk, DAC_MOSI => dac_mosi, DAC_SS => dac_ss);

  stim: process
  begin
    wait for 200 ns;
    rstn <= '1';
    wait for 500 ns;

    -- envelope params: fast attack/decay/release, mid sustain
    send_midi_message(midi_clk, midi_data, midi_startn, create_midi_message(PARAM, 73, 100));
    send_midi_message(midi_clk, midi_data, midi_startn, create_midi_message(PARAM, 75, 100));
    send_midi_message(midi_clk, midi_data, midi_startn, create_midi_message(PARAM, 70, 64));
    send_midi_message(midi_clk, midi_data, midi_startn, create_midi_message(PARAM, 72, 100));

    -- note on, should land on voice 0
    send_midi_message(midi_clk, midi_data, midi_startn, create_midi_message(NOTE_ON, 60, 110));

    wait;
  end process;

  check: process
    variable ch_a_val : std_logic_vector(15 downto 0);
    variable first_a  : unsigned(11 downto 0);
    variable later_a  : unsigned(11 downto 0) := (others => '0');
    variable got_rise : boolean := false;
  begin
    -- gate must go high shortly after the note-on message (~9 frames of MIDI)
    wait until env_gate0 = '1' for 5 ms;
    assert env_gate0 = '1' report "env_gate0 never went high after note-on" severity error;

    -- decode I2C channel A across several frames, envelope must rise
    for frame in 1 to 8 loop
      wait until falling_edge(dac_mosi) and to_x01(dac_sclk) = '1'; -- START
      for i in 7 downto 0 loop
        wait until rising_edge(dac_sclk);
      end loop;
      wait until rising_edge(dac_sclk); -- ack
      ch_a_val := (others => '0');
      for i in 15 downto 8 loop
        wait until rising_edge(dac_sclk);
        ch_a_val(i) := to_x01(dac_mosi);
      end loop;
      wait until rising_edge(dac_sclk); -- ack
      for i in 7 downto 0 loop
        wait until rising_edge(dac_sclk);
        ch_a_val(i) := to_x01(dac_mosi);
      end loop;
      if frame = 1 then
        first_a := unsigned(ch_a_val(11 downto 0));
      else
        later_a := unsigned(ch_a_val(11 downto 0));
        if later_a > first_a then
          got_rise := true;
        end if;
      end if;
      -- skip remaining 6 bytes + acks of this frame
      for b in 1 to 6 loop
        wait until rising_edge(dac_sclk);
        for i in 7 downto 0 loop
          wait until rising_edge(dac_sclk);
        end loop;
      end loop;
      wait until rising_edge(dac_sclk); -- final ack
    end loop;

    assert got_rise
      report "DAC channel A never rose after note-on (first=" &
             integer'image(to_integer(first_a)) & " later=" &
             integer'image(to_integer(later_a)) & ")"
      severity error;

    report "synth_top_tb PASSED" severity note;
    done <= true;
    wait;
  end process;

end architecture;
