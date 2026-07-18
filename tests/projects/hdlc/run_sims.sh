#!/bin/sh
# Compile the full design + testbenches with GHDL and run every testbench.
# Run from this directory:  ./run_sims.sh [ghdl-binary]
# rtl/gen/* are outputs of `gen_hdlc.py` + `hdl-compose codegen` — regenerate
# after editing the .hdlc files, never hand-edit them.
set -e
GHDL=${1:-ghdl}
FLAGS="--std=08 --workdir=build -Pbuild"
mkdir -p build

# loot: LUT packages used by the MIDI front end
$GHDL -a $FLAGS --work=loot rtl/loot/midi_lut_pkg.vhd rtl/loot/cc_lut_pkg.vhd

# work: dependency order matters (register chain bottom-up, i2s_tx before manager)
$GHDL -a $FLAGS \
  rtl/registers/ranger_mapper.vhd \
  rtl/registers/dflipflop.vhd \
  rtl/registers/n_parallel_register.vhd \
  rtl/registers/addressed_pl_reg.vhd \
  rtl/registers/map_reg.vhd \
  rtl/registers/mapped_reg.vhd \
  rtl/midi/midi_interface.vhd \
  rtl/midi/midi_processor.vhd \
  rtl/midi/edge_detect.vhd \
  rtl/oscillator/phase_incre.vhd \
  rtl/oscillator/square_wave.vhd \
  rtl/oscillator/sawtooth.vhd \
  rtl/oscillator/mux.vhd \
  rtl/oscillator/volume_scaler.vhd \
  rtl/audio/bitwise_vec_op.vhd \
  rtl/audio/moving_average.vhd \
  rtl/audio/output_stage.vhd \
  rtl/audio/i2s_tx.vhd \
  rtl/audio/i2s_manager.vhd \
  rtl/audio/clock_obuf.vhd \
  rtl/cv/enveloper.vhd \
  rtl/cv/lfo.vhd \
  rtl/cv/cv_mix.vhd \
  rtl/cv/i2c_cv_dac.vhd \
  rtl/cv/axis_cdc.vhd \
  rtl/cv/cv_modulation.vhd \
  rtl/gen/voice_chain.vhd \
  sim/selectio_sim.vhd \
  rtl/gen/synth_top.vhd \
  sim/midi_interface_tb_pkg.vhd \
  sim/enveloper_tb.vhd \
  sim/lfo_tb.vhd \
  sim/i2c_cv_dac_tb.vhd \
  sim/cv_modulation_tb.vhd \
  sim/axis_cdc_tb.vhd \
  sim/midi_processor_tb.vhd \
  sim/synth_top_tb.vhd

for tb in enveloper_tb lfo_tb i2c_cv_dac_tb cv_modulation_tb axis_cdc_tb midi_processor_tb synth_top_tb; do
  echo "=== $tb ==="
  # legacy midi_processor_tb has no std.env.finish; bound every run instead
  $GHDL elab-run $FLAGS "$tb" --assert-level=error --stop-time=100ms
done
echo "ALL TESTBENCHES PASSED"
