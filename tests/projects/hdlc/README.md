# synth_top — hdl-compose project

Standalone extract of the 2-voice synth's actually-used sources from the
Vivado project, restructured for hdl-compose + GHDL. No Vivado boilerplate.

## Layout

```
voice_chain.hdlc   one voice signal chain (wave select, osc, VCA env, audio out)
synth_top.hdlc     full top: MIDI front end, 2x voice_chain, VCF env + LFO + mix, I2C CV DAC
gen_hdlc.py        single source of truth — writes both .hdlc files. Edit this, not the JSON.
run_sims.sh        compile everything with GHDL and run all 7 testbenches
rtl/
  loot/            MIDI/CC LUT packages (compiled into library `loot`)
  registers/       d_ff -> n_parallel_register -> addressed_pl_reg -> map_reg/mapped_reg, range_mapper
  midi/            midi_interface (deserializer), midi_processor (voice alloc + param bus), edge_detect
  oscillator/      phase_incre, square_wave, sawtooth, mux, volume_scaler
  audio/           bitwise_vec_op, moving_average, output_stage, i2s_tx/i2s_manager,
                   clock_obuf, clocked_data_out.v (selectio, Verilog — Vivado build only)
  cv/              enveloper (ADSR), lfo, cv_mix, i2c_cv_dac (MCP4728), axis_cdc (gray-code FIFO),
                   cv_modulation (alternative single-module bundle, unused by synth_top)
  gen/             GENERATED voice_chain.vhd + synth_top.vhd — never hand-edit
sim/               testbenches + selectio_sim (behavioral, GHDL-only, never add to Vivado)
```

## Workflow

Regenerate after any structural change (run from this directory):

```sh
python3 gen_hdlc.py
hdl-compose codegen voice_chain.hdlc -o rtl/gen/voice_chain.vhd
hdl-compose codegen synth_top.hdlc   -o rtl/gen/synth_top.vhd
./run_sims.sh            # or ./run_sims.sh /path/to/ghdl
```

`library_paths` in the .hdlc files are relative to this directory — run
hdl-compose (CLI or GUI) with cwd here.

Toolchain checks (oss-cad-suite on PATH):

```sh
hdl-compose check synth_top.hdlc     # ghdl elaboration of the generated top
hdl-compose synth synth_top.hdlc     # yosys synthesizability + cell stats
```

`*.toolchain.json` sidecars tell those commands what the .hdlc files can't:
the `loot` LUT packages (analyzed as a named library), leaf dependencies of
the library modules (register chain, i2s_tx, edge_detect), the GHDL-only
`selectio_sim.vhd`, the Vivado-only `clocked_data_out.v` exclusion, and
`--latches` for the midi_processor state decode.

Net convention reminder: an instance OUTPUT feeding others references
itself in its port_map; INPUTS reference the driving instance/port.
Mutual references silently split the net in two.

## CC / param address map

| Param            | Addr |
|------------------|------|
| VCA env A/D/S/R  | 73 / 75 / 70 / 72 |
| VCF env A/D/S/R  | 80 / 81 / 82 / 83 |
| Wave select      | 84 |
| LFO rate / depth | 85 / 86 |

CV DAC: MCP4728 on PMOD JA (DAC_SCLK=SCL, DAC_MOSI=SDA open-drain, DAC_SS=LDAC
held low). Channels A/B = voice envelopes, C = VCF env + LFO mix, D unused.
