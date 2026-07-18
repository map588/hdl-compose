
library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;

-- Structural top for the CV/modulation section: one ADSR per voice (VCA CVs),
-- a VCF ADSR summed with a triangle LFO, all serialized to the MCP4728 quad
-- DAC over I2C.
--
--   MCP4728 ch A = voice 0 VCA CV     (envelope, gate = note_states(0))
--   MCP4728 ch B = voice 1 VCA CV     (envelope, gate = note_states(1))
--   MCP4728 ch C = VCF CV             (envelope [gate = either voice] + LFO*depth)
--   MCP4728 ch D = unused (written 0)
--
-- Param-bus addresses (from cc_lut_pkg):
--   VCA ADSR  A/D/S/R = 73/75/70/72  (shared by both voices)
--   VCF ADSR  A/D/S/R = 80/81/82/83
--   LFO rate = 85, LFO depth = 86
--
-- Block-design hookup (single module_ref cell on the Vivado machine):
--   clk         <- same 100 MHz net as midi_processor_0/clk (FCLK_CLK1 net)
--   rst_n       <- proc_sys_reset_0/peripheral_aresetn
--   note_states <- midi_processor_0/note_states
--   triggers    <- midi_processor_0/triggers
--   param_*     <- midi_processor_0/param_write / param_number / param_value
--   scl  -> DAC_SCLK port (Y17), sda -> DAC_MOSI port (Y19, must become inout),
--   ldac -> DAC_SS port (Y18). Delete the old nets from processing_system7_0
--   SPI0_*_O to those ports first.

entity cv_modulation is
  generic (
    CLKS_PER_MS : integer := 104167
  );
  port (
    clk         : in    std_logic;
    rst_n       : in    std_logic;
    note_states : in    std_logic_vector(1 downto 0);
    triggers    : in    std_logic_vector(1 downto 0);
    param_write : in    std_logic;
    param_number: in    std_logic_vector(6 downto 0);
    param_value : in    std_logic_vector(6 downto 0);
    env_gates   : out   std_logic_vector(1 downto 0); -- envelope-active flags (debug/LEDs)
    scl         : out   std_logic;
    sda         : inout std_logic;
    ldac        : out   std_logic
  );
end entity;

architecture Structural of cv_modulation is

  component enveloper is
    generic (
      ATTACK_ADDR   : integer range 0 to 127 := 73;
      DECAY_ADDR    : integer range 0 to 127 := 75;
      SUSTAIN_ADDR  : integer range 0 to 127 := 70;
      RELEASE_ADDR  : integer range 0 to 127 := 72;
      CLKS_PER_TICK : integer               := 104167
    );
    port (
      clk         : in  std_logic;
      rst_n       : in  std_logic;
      gate        : in  std_logic;
      trig        : in  std_logic;
      param_write : in  std_logic;
      param_addr  : in  std_logic_vector(6 downto 0);
      param_data  : in  std_logic_vector(6 downto 0);
      gate_out    : out std_logic;
      env_out     : out std_logic_vector(11 downto 0)
    );
  end component;

  component lfo is
    generic (
      RATE_ADDR     : integer range 0 to 127 := 85;
      CLKS_PER_TICK : integer               := 10417
    );
    port (
      clk         : in  std_logic;
      rst_n       : in  std_logic;
      param_write : in  std_logic;
      param_addr  : in  std_logic_vector(6 downto 0);
      param_data  : in  std_logic_vector(6 downto 0);
      lfo_out     : out std_logic_vector(11 downto 0)
    );
  end component;

  component cv_mix is
    generic (
      DEPTH_ADDR : integer range 0 to 127 := 86
    );
    port (
      clk         : in  std_logic;
      rst_n       : in  std_logic;
      param_write : in  std_logic;
      param_addr  : in  std_logic_vector(6 downto 0);
      param_data  : in  std_logic_vector(6 downto 0);
      env_in      : in  std_logic_vector(11 downto 0);
      lfo_in      : in  std_logic_vector(11 downto 0);
      cv_out      : out std_logic_vector(11 downto 0)
    );
  end component;

  component i2c_cv_dac is
    generic (
      CLK_DIV  : integer := 260;
      DEV_ADDR : std_logic_vector(6 downto 0) := "1100000"
    );
    port (
      clk   : in    std_logic;
      rst_n : in    std_logic;
      ch_a  : in    std_logic_vector(11 downto 0);
      ch_b  : in    std_logic_vector(11 downto 0);
      ch_c  : in    std_logic_vector(11 downto 0);
      scl   : out   std_logic;
      sda   : inout std_logic;
      ldac  : out   std_logic
    );
  end component;

  signal env_vca0 : std_logic_vector(11 downto 0);
  signal env_vca1 : std_logic_vector(11 downto 0);
  signal env_vcf  : std_logic_vector(11 downto 0);
  signal lfo_val  : std_logic_vector(11 downto 0);
  signal vcf_cv   : std_logic_vector(11 downto 0);

  signal vcf_gate : std_logic;
  signal vcf_trig : std_logic;

begin

  vcf_gate <= note_states(0) or note_states(1);
  vcf_trig <= triggers(0) or triggers(1);

  env_voice0: enveloper
    generic map (
      ATTACK_ADDR => 73, DECAY_ADDR => 75, SUSTAIN_ADDR => 70, RELEASE_ADDR => 72,
      CLKS_PER_TICK => CLKS_PER_MS)
    port map (
      clk => clk, rst_n => rst_n,
      gate => note_states(0), trig => triggers(0),
      param_write => param_write, param_addr => param_number, param_data => param_value,
      gate_out => env_gates(0), env_out => env_vca0);

  env_voice1: enveloper
    generic map (
      ATTACK_ADDR => 73, DECAY_ADDR => 75, SUSTAIN_ADDR => 70, RELEASE_ADDR => 72,
      CLKS_PER_TICK => CLKS_PER_MS)
    port map (
      clk => clk, rst_n => rst_n,
      gate => note_states(1), trig => triggers(1),
      param_write => param_write, param_addr => param_number, param_data => param_value,
      gate_out => env_gates(1), env_out => env_vca1);

  env_filter: enveloper
    generic map (
      ATTACK_ADDR => 80, DECAY_ADDR => 81, SUSTAIN_ADDR => 82, RELEASE_ADDR => 83,
      CLKS_PER_TICK => CLKS_PER_MS)
    port map (
      clk => clk, rst_n => rst_n,
      gate => vcf_gate, trig => vcf_trig,
      param_write => param_write, param_addr => param_number, param_data => param_value,
      gate_out => open, env_out => env_vcf);

  lfo_inst: lfo
    generic map (RATE_ADDR => 85, CLKS_PER_TICK => CLKS_PER_MS / 10)
    port map (
      clk => clk, rst_n => rst_n,
      param_write => param_write, param_addr => param_number, param_data => param_value,
      lfo_out => lfo_val);

  vcf_mix: cv_mix
    generic map (DEPTH_ADDR => 86)
    port map (
      clk => clk, rst_n => rst_n,
      param_write => param_write, param_addr => param_number, param_data => param_value,
      env_in => env_vcf, lfo_in => lfo_val, cv_out => vcf_cv);

  dac: i2c_cv_dac
    generic map (CLK_DIV => 260, DEV_ADDR => "1100000")
    port map (
      clk => clk, rst_n => rst_n,
      ch_a => env_vca0, ch_b => env_vca1, ch_c => vcf_cv,
      scl => scl, sda => sda, ldac => ldac);

end architecture;
