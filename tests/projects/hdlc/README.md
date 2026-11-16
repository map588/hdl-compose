# synth_top — hdl-compose design

Standalone extract of the 2-voice synth's actually-used sources from the
Vivado project, restructured for hdl-compose + GHDL. No Vivado boilerplate.

## Layout

```
voice_chain.hdlc   one voice signal chain (wave select, osc, VCA env, audio out)
synth_top.hdlc     full top: MIDI front end, 2x voice_chain, VCF env + LFO + mix, I2C CV DAC
gen_hdlc.py        writes both .hdlc files at format version 4. See the note below before you run it.
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

`gen_hdlc.py` gives the tracked `voice_chain.hdlc` exactly. The tracked
`synth_top.hdlc` was saved from the GUI after the script wrote it (format
version 5): it has no `selectio0` / `selectio1` instances, the `i2s`
outputs go to the top-level ports directly, and it has two more library
paths. A run of the script replaces that file with the older design.

Regenerate after any structural change (run from this directory):

```sh
python3 gen_hdlc.py
hdlc codegen voice_chain.hdlc -o rtl/gen/voice_chain.vhd
hdlc codegen synth_top.hdlc   -o rtl/gen/synth_top.vhd
./run_sims.sh            # or ./run_sims.sh /path/to/ghdl
```

`library_paths` in the .hdlc files are relative to this directory. The
commands here use paths relative to it, but `hdlc` also takes a design
path from another directory.

Toolchain checks (oss-cad-suite on PATH):

```sh
hdlc check synth_top.hdlc     # ghdl elaboration of the generated top
hdlc synth synth_top.hdlc     # yosys synthesizability + cell stats
```

`*.toolchain.json` sidecars tell those commands what the .hdlc files can't:
the `loot` LUT packages (analyzed as a named library), leaf dependencies of
the library modules (register chain, i2s_tx, edge_detect), the GHDL-only
`selectio_sim.vhd`, the Vivado-only `clocked_data_out.v` exclusion, and
`--latches` for the midi_processor state decode.

`synth_top` uses the design `voice_chain`. A tool run of `synth_top` takes
the `loot` packages and the leaf sources from `voice_chain.toolchain.json`,
so `synth_top.toolchain.json` holds only what `synth_top` adds: `i2s_tx`,
`selectio_sim.vhd`, the exclusion and the `--latches` flag.

Net convention: a `port_map` entry states that two pins are connected, and
its direction has no effect. `hdlc` merges all pins that reference each
other into one net, so an output needs no reference to itself (see
`docs/FORMAT.md`).

## CC / param address map

| Param            | Addr |
|------------------|------|
| VCA env A/D/S/R  | 73 / 75 / 70 / 72 |
| VCF env A/D/S/R  | 80 / 81 / 82 / 83 |
| Wave select      | 84 |
| LFO rate / depth | 85 / 86 |

CV DAC: MCP4728 on PMOD JA (DAC_SCLK=SCL, DAC_MOSI=SDA open-drain, DAC_SS=LDAC
held low). Channels A/B = voice envelopes, C = VCF env + LFO mix, D unused.
