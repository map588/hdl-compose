# The `.hdlc` project format

A `.hdlc` file is JSON: a `version` field plus the schematic, flattened into
one object. The authoritative machine-readable schema is
[`hdlc.schema.json`](hdlc.schema.json) (regenerate with `hdl-compose schema`).
For a working starting point, run:

```sh
hdl-compose new my_top -l vhdl --example
```

which creates `my_top.hdlc` plus two example HDL sources, wired together.

## Nets and connectivity

Instance ports connect through `port_map` entries. Each entry maps a port of
the instance to a **reference**:

| JSON value                                   | Meaning                       |
|----------------------------------------------|-------------------------------|
| `{"InstancePort": ["u_a", "dout"]}`          | pin `u_a.dout`                |
| `{"TopPort": "clk"}`                         | top-level port `clk`          |
| `{"InstancePortSlice": ["u_a","dout",{"Bit":3}]}` | bit 3 of `u_a.dout`      |
| `{"TopPortSlice": ["bus",{"Range":{"high":7,"low":0}}]}` | `bus[7:0]`     |
| `{"Constant": "'0'"}`                        | literal tie (emitted verbatim)|
| `null`                                       | explicitly unconnected        |

**A reference is a connectivity statement, not a naming convention.**
`hdl-compose` merges all mutually-reachable pins into one net, so
`B.din => A.dout` and `A.dout => B.din` describe the same wire — the
direction you write it never matters, and you never need self-references.

Each resolved net gets one signal in the generated HDL, named:

1. the net's **alias**, if one is set (`aliases` map, keyed by any pin of the
   net in string form, e.g. `"u_a.dout": "data_bus"`), else
2. `<top port>_s` when the net touches a top-level port, else
3. `<driver instance>_<driver port>`.

Validation is net-level: a net with **two or more hard drivers** (instance
outputs / top inputs, InOut excluded) is an error; a net with **no driver**
is a warning.

## Constants

`{"Constant": "<literal>"}` ties an input to a literal, emitted verbatim in
the target language — write `'0'`, `"0101"`, `x"AB"` for VHDL and `1'b0`,
`8'hFF` for SystemVerilog. Constants are direct associations: no signal is
declared. Tying an output to a constant is an error.

Unconnected inputs stay warnings, but note VHDL rejects `open` on an input
without a default value — tie it or connect it.

## Generics

`generic_map` values are free strings, emitted verbatim, with two rules:

- **String generics**: a bare word for a string-typed generic is quoted for
  you (`"OP": "OR"` emits `OP => "OR"`). Already-quoted values pass through.
- **Top-generic passthrough**: a value naming a top-level generic
  (`"DIV": "CLK_DIV"`) is a reference and is never quoted — use this to
  forward top generics into instances.

## CLI exit codes

`hdl-compose validate` exits 0 when clean, 1 with warnings only, 2 with
errors (or when the project fails to load/parse).
