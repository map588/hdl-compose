library IEEE;
  use IEEE.STD_LOGIC_1164.all;
  use IEEE.NUMERIC_STD.all;
  use IEEE.MATH_REAL.all;


package cc_lut_pkg is
  function int_to_slv(int_bits : natural; i : integer) return std_logic_vector;


  type cc_lut_type is array (0 to 21) of std_logic_vector(6 downto 0);
  type ADSR_ENUM is (VCA, VCF, MODIFIER, UNKNOWN);
  
  type ADSR is (A, D, S, R);
  type ADSR_indices is array (ADSR) of integer;


  constant cc_lut : cc_lut_type;

  constant wave_cc_idx   : integer;
  constant volume_cc_idx : integer;

  constant vca_attack_cc_idx  : integer;
  constant vca_decay_cc_idx   : integer;
  constant vca_sustain_cc_idx : integer;
  constant vca_release_cc_idx : integer;

  constant vcf_attack_cc_idx  : integer;
  constant vcf_decay_cc_idx   : integer;
  constant vcf_sustain_cc_idx : integer;
  constant vcf_release_cc_idx : integer;

  constant mod_attack_cc_idx  : integer;
  constant mod_decay_cc_idx   : integer;
  constant mod_sustain_cc_idx : integer;
  constant mod_release_cc_idx : integer;

  constant dig_filter_cc_idx : integer;

  constant filter_freq_cc_idx : integer;
  constant filter_res_cc_idx  : integer;

  constant fx1_cc_idx : integer;
  constant fx2_cc_idx : integer;
  constant fx3_cc_idx : integer;
  constant fx4_cc_idx : integer;
  constant fx5_cc_idx : integer;

  ----------------------------------------------------------------------------------

  constant vol_cc_addr : integer;
  constant wav_cc_addr : integer;

  constant vca_attack_cc_addr  : integer;
  constant vca_decay_cc_addr   : integer;
  constant vca_sustain_cc_addr : integer;
  constant vca_release_cc_addr : integer;

  constant vcf_attack_cc_addr  : integer;
  constant vcf_decay_cc_addr   : integer;
  constant vcf_sustain_cc_addr : integer;
  constant vcf_release_cc_addr : integer;

  constant mod_attack_cc_addr  : integer;
  constant mod_decay_cc_addr   : integer;
  constant mod_sustain_cc_addr : integer;
  constant mod_release_cc_addr : integer;

  constant dig_filter_cc_addr : integer;

  constant filter_freq_cc_addr : integer;
  constant filter_res_cc_addr  : integer;

  constant fx1_cc_addr : integer;
  constant fx2_cc_addr : integer;
  constant fx3_cc_addr : integer;
  constant fx4_cc_addr : integer;
  constant fx5_cc_addr : integer;

----------------------------------------------------------------------------------

  constant wave_cc_slv   : std_logic_vector(6 downto 0);
  constant volume_cc_slv : std_logic_vector(6 downto 0);

  constant vca_attack_cc_slv  : std_logic_vector(6 downto 0);
  constant vca_decay_cc_slv   : std_logic_vector(6 downto 0);
  constant vca_sustain_cc_slv : std_logic_vector(6 downto 0);
  constant vca_release_cc_slv : std_logic_vector(6 downto 0);

  constant vcf_attack_cc_slv  : std_logic_vector(6 downto 0);
  constant vcf_decay_cc_slv   : std_logic_vector(6 downto 0);
  constant vcf_sustain_cc_slv : std_logic_vector(6 downto 0);
  constant vcf_release_cc_slv : std_logic_vector(6 downto 0);

  constant mod_attack_cc_slv  : std_logic_vector(6 downto 0);
  constant mod_decay_cc_slv   : std_logic_vector(6 downto 0);
  constant mod_sustain_cc_slv : std_logic_vector(6 downto 0);
  constant mod_release_cc_slv : std_logic_vector(6 downto 0);

  constant dig_filter_cc_slv : std_logic_vector(6 downto 0);

  constant filter_freq_cc_slv : std_logic_vector(6 downto 0);
  constant filter_res_cc_slv  : std_logic_vector(6 downto 0);

  constant fx1_cc_slv : std_logic_vector(6 downto 0);
  constant fx2_cc_slv : std_logic_vector(6 downto 0);
  constant fx3_cc_slv : std_logic_vector(6 downto 0);
  constant fx4_cc_slv : std_logic_vector(6 downto 0);
  constant fx5_cc_slv : std_logic_vector(6 downto 0);

  

end package;

package body cc_lut_pkg is
  function int_to_slv(int_bits : natural; i : integer) return std_logic_vector is
    variable result : std_logic_vector(int_bits - 1 downto 0);
  begin
    result := std_logic_vector(to_unsigned(i, int_bits));
    return result;
  end function;

  constant vol_cc_addr : integer := 7;
  constant wav_cc_addr : integer := 84;

  constant vca_attack_cc_addr  : integer := 73;
  constant vca_decay_cc_addr   : integer := 75;
  constant vca_sustain_cc_addr : integer := 70;
  constant vca_release_cc_addr : integer := 72;

  constant vcf_attack_cc_addr  : integer := 80;
  constant vcf_decay_cc_addr   : integer := 81;
  constant vcf_sustain_cc_addr : integer := 82;
  constant vcf_release_cc_addr : integer := 83;

  constant mod_attack_cc_addr  : integer := 85;
  constant mod_decay_cc_addr   : integer := 86;
  constant mod_sustain_cc_addr : integer := 87;
  constant mod_release_cc_addr : integer := 88;

  constant filter_freq_cc_addr : integer := 71;
  constant filter_res_cc_addr  : integer := 74;

  constant dig_filter_cc_addr : integer := 96;
  
  constant fx1_cc_addr : integer := 91;
  constant fx2_cc_addr : integer := 92;
  constant fx3_cc_addr : integer := 93;
  constant fx4_cc_addr : integer := 94;
  constant fx5_cc_addr : integer := 95;

  constant cc_lut : cc_lut_type := (
    -- Oscillator values
    int_to_slv(7, vol_cc_addr), --volume
    int_to_slv(7, wav_cc_addr), --waveform

        -- FX values
    int_to_slv(7, fx1_cc_addr),  --fx 1
    int_to_slv(7, fx2_cc_addr),  --fx 2
    int_to_slv(7, fx3_cc_addr),  --fx 3
    int_to_slv(7, fx4_cc_addr),  --fx 4
    int_to_slv(7, fx5_cc_addr),  --fx 5
    -- VCA ADSR
    int_to_slv(7, vca_attack_cc_addr),
    int_to_slv(7, vca_decay_cc_addr),
    int_to_slv(7, vca_sustain_cc_addr),
    int_to_slv(7, vca_release_cc_addr),

    -- VCF ADSR
    int_to_slv(7, vcf_attack_cc_addr),
    int_to_slv(7, vcf_decay_cc_addr),
    int_to_slv(7, vcf_sustain_cc_addr),
    int_to_slv(7, vcf_release_cc_addr),

    -- mod ADSR
    int_to_slv(7, mod_attack_cc_addr),
    int_to_slv(7, mod_decay_cc_addr),
    int_to_slv(7, mod_sustain_cc_addr),
    int_to_slv(7, mod_release_cc_addr),

    -- Filter values
    int_to_slv(7, filter_freq_cc_addr),
    int_to_slv(7, filter_res_cc_addr),

    -- Digital filter
    int_to_slv(7, dig_filter_cc_addr)
  );

	constant volume_cc_idx : integer := 0;
  constant wave_cc_idx   : integer := 1;

	constant fx1_cc_idx  : integer := 2;
  constant fx2_cc_idx  : integer := 3;
  constant fx3_cc_idx  : integer := 4;
  constant fx4_cc_idx  : integer := 5;
  constant fx5_cc_idx  : integer := 6;

  constant vca_attack_cc_idx  : integer := 7;
  constant vca_decay_cc_idx   : integer := 8;
  constant vca_sustain_cc_idx : integer := 9;
  constant vca_release_cc_idx : integer := 10;

  constant vcf_attack_cc_idx  : integer := 11;
  constant vcf_decay_cc_idx   : integer := 12;
  constant vcf_sustain_cc_idx : integer := 13;
  constant vcf_release_cc_idx : integer := 14;

  constant mod_attack_cc_idx  : integer := 15;
  constant mod_decay_cc_idx   : integer := 16;
  constant mod_sustain_cc_idx : integer := 17;
  constant mod_release_cc_idx : integer := 18;

  constant filter_freq_cc_idx : integer := 19;
  constant filter_res_cc_idx  : integer := 20;

  constant dig_filter_cc_idx : integer := 21;




  constant wave_cc_slv   : std_logic_vector(6 downto 0) := cc_lut(wave_cc_idx);
  constant volume_cc_slv : std_logic_vector(6 downto 0) := cc_lut(volume_cc_idx);

  constant vca_attack_cc_slv  : std_logic_vector(6 downto 0) := cc_lut(vca_attack_cc_idx);
  constant vca_decay_cc_slv   : std_logic_vector(6 downto 0) := cc_lut(vca_decay_cc_idx);
  constant vca_sustain_cc_slv : std_logic_vector(6 downto 0) := cc_lut(vca_sustain_cc_idx);
  constant vca_release_cc_slv : std_logic_vector(6 downto 0) := cc_lut(vca_release_cc_idx);

  constant vcf_attack_cc_slv  : std_logic_vector(6 downto 0) := cc_lut(vcf_attack_cc_idx);
  constant vcf_decay_cc_slv   : std_logic_vector(6 downto 0) := cc_lut(vcf_decay_cc_idx);
  constant vcf_sustain_cc_slv : std_logic_vector(6 downto 0) := cc_lut(vcf_sustain_cc_idx);
  constant vcf_release_cc_slv : std_logic_vector(6 downto 0) := cc_lut(vcf_release_cc_idx);

  constant mod_attack_cc_slv  : std_logic_vector(6 downto 0) := cc_lut(mod_attack_cc_idx);
  constant mod_decay_cc_slv   : std_logic_vector(6 downto 0) := cc_lut(mod_decay_cc_idx);
  constant mod_sustain_cc_slv : std_logic_vector(6 downto 0) := cc_lut(mod_sustain_cc_idx);
  constant mod_release_cc_slv : std_logic_vector(6 downto 0) := cc_lut(mod_release_cc_idx);

  constant dig_filter_cc_slv : std_logic_vector(6 downto 0) := cc_lut(dig_filter_cc_idx);

  constant filter_freq_cc_slv : std_logic_vector(6 downto 0) := cc_lut(filter_freq_cc_idx);
  constant filter_res_cc_slv  : std_logic_vector(6 downto 0) := cc_lut(filter_res_cc_idx);

  constant fx1_cc_slv : std_logic_vector(6 downto 0) := cc_lut(fx1_cc_idx);
  constant fx2_cc_slv : std_logic_vector(6 downto 0) := cc_lut(fx2_cc_idx);
  constant fx3_cc_slv : std_logic_vector(6 downto 0) := cc_lut(fx3_cc_idx);
  constant fx4_cc_slv : std_logic_vector(6 downto 0) := cc_lut(fx4_cc_idx);
  constant fx5_cc_slv : std_logic_vector(6 downto 0) := cc_lut(fx5_cc_idx);

end package body;
