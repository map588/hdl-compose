library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

  -- This component deserializes the MIDI input stream and outputs 16-bit words

entity midi_interface is
  port (
    sys_clk     : in  std_logic;
    sys_rstn    : in  std_logic;
    midi_clk    : in  std_logic;
    midi_sdata  : in  std_logic;
    midi_startn : in  std_logic;
    cpu_ack     : in  std_logic := '0';
    midi_output : out std_logic_vector(15 downto 0);
    interrupt   : out std_logic;       -- Towards CPU
    -- No default value: ghdl-yosys-plugin cannot translate the $ioutput
    -- gate GHDL emits for out-port defaults; reset drives it '0' anyway.
    update      : out std_logic -- Towards PL
  );
  attribute ASYNC_REG : string;
  attribute ASYNC_REG of update  : signal is "TRUE";
end entity;

architecture Behavioral of midi_interface is
  -- IOB registers (preserve these)
  signal midi_data_iob   : std_logic;
  signal midi_startn_iob : std_logic;
  signal midi_clk_iob    : std_logic;
  signal midi_start_iob  : std_logic;
  signal midi_clk_hbuf : std_logic;

  -- Core data path
  signal shift_reg  : std_logic_vector(15 downto 0) := (others => '0');
  signal bit_count  : unsigned(4 downto 0) := (others => '0');
  signal data_valid : std_logic := '0';

  -- data_valid crosses from the midi_clk domain and stays high once the MIDI
  -- clock stops at end of frame; sync it and act on the rising edge only, so
  -- every frame (not just the first) raises update/interrupt.
  signal dv_meta : std_logic := '0';
  signal dv_sync : std_logic := '0';
  signal dv_prev : std_logic := '0';

  attribute ASYNC_REG of dv_meta : signal is "TRUE";
  attribute ASYNC_REG of dv_sync : signal is "TRUE";

  -- Critical timing attributes
  attribute IOB                                     : string;
  attribute IOB of midi_data_iob                    : signal is "TRUE";
  attribute IOB of midi_startn_iob                  : signal is "TRUE";
  attribute IOB of midi_clk_iob                     : signal is "TRUE";

  attribute CLOCK_DEDICATED_ROUTE                   : string;
  attribute CLOCK_DEDICATED_ROUTE of midi_clk_iob   : signal is "TRUE";
  attribute CLOCK_DEDICATED_ROUTE of midi_clk_hbuf  : signal is "TRUE";
begin
  -- Data capture in IO Buffer Registers
  process (sys_clk)
  begin
    if rising_edge(sys_clk) then
      midi_data_iob   <= midi_sdata;
      midi_startn_iob <= midi_startn;
      midi_clk_iob    <= midi_clk;
    end if;
  end process;

  -- Clock-enable buffered clock (was BUFHCE): the enable gates the clock off,
  -- holding the output low while CE is deasserted, as BUFHCE does with CE = 0.
  midi_clk_hbuf <= midi_clk_iob when midi_startn_iob = '1' else '0';

  -- High-speed data path
  capture_proc: process (midi_clk_hbuf)
  begin
    if rising_edge(midi_clk_hbuf) then
      if midi_startn_iob = '1' then -- Active frame
        shift_reg       <= shift_reg(14 downto 0) & midi_data_iob;

        if bit_count = 15 then
          midi_output   <= shift_reg(14 downto 0) & midi_data_iob;
          bit_count     <= (others => '0');
          data_valid    <= '1';
        else
          bit_count     <= bit_count + 1;
          data_valid    <= '0';
        end if;
      end if;
    end if;
  end process;

  -- Status generation
  status_proc: process (sys_clk)
  begin
    if rising_edge(sys_clk) then
      if sys_rstn = '0' then
        interrupt <= '0';
        update    <= '0';
        dv_meta   <= '0';
        dv_sync   <= '0';
        dv_prev   <= '0';
      else
        dv_meta <= data_valid;
        dv_sync <= dv_meta;
        dv_prev <= dv_sync;

        if dv_sync = '1' and dv_prev = '0' then
          interrupt <= '1';
          update    <= '1';
        elsif cpu_ack = '1' then
          interrupt <= '0';
          update    <= '0';
        end if;
      end if;
    end if;
  end process;

end architecture;
