# Write an `hdlc` project: a guide for agents

`hdlc guide` prints this guide, then the `.hdlc` format reference.

## What `hdlc` does

- You write the leaf modules in VHDL or SystemVerilog. `hdlc` writes no logic.
- A `.hdlc` file is JSON. It names instances of the leaf modules and connects
  their ports.
- `hdlc codegen` writes one structural top-level module from the `.hdlc` file.
- Do not edit the generated file. Edit the `.hdlc` file, then generate again.
- A project has one language. Write the leaf modules in that language.

## Procedure

1. Write the leaf modules.
2. Run `hdlc parse <file>` on each source file. The output shows the module
   name, the generics, and the ports as `hdlc` read them. Use these names,
   with the same spelling, in `module_ref`, `generic_map` and `port_map`.
   The output uses VHDL type names for SystemVerilog sources too.
3. Write `<top>.hdlc`. Copy the example below, or run
   `hdlc new <top> -l vhdl|sv`, which writes an empty `<top>.hdlc` into the
   current directory.
4. Run `hdlc validate <top>.hdlc`. It reports a name that does not exist (a
   module, a port, a generic, an instance, a top-level port), a width
   mismatch, a net with more than one driver, and a port with no connection.
   Exit code 0 is clean, 1 is warnings only, 2 is errors. Correct the file
   until no error stays, and until each warning that stays is for an output
   that you do not use.
5. Run `hdlc codegen <top>.hdlc -o <top>.vhd` (or `.sv`).
6. Run `hdlc check <top>.hdlc`. It elaborates the generated top level and the
   leaf modules with ghdl (VHDL) or verilator (SystemVerilog). It finds what
   only the HDL tool can know, such as an error in a leaf module. Exit code
   0 is clean, 1 is tool errors, 2 is a missing tool or a codegen failure.

`hdlc validate` calculates a width from a generic when the generic has an
integer value. For a top-level generic it uses the default value. It makes no
check for a width that it cannot calculate: a generic with no integer value,
a `Record` or `Other` type. ghdl can accept a mismatch of such a width in
`hdlc check` and report it only when the simulation starts (`hdlc sim`).

Other commands: `hdlc inspect <top>.hdlc` prints a summary, `hdlc synth` runs
yosys, `hdlc sim` runs `<top>_tb`, and `hdlc schema` prints the JSON Schema.
`hdlc <command> --help` gives the arguments.

## Complete example

The leaf modules are in `pulse_gen.vhd` and `led_driver.vhd`, in the same
directory as the project file. `hdlc parse` shows these interfaces:

```text
module pulse_gen
  generics:
    WIDTH : integer := 8
    DIV : integer := 1000
  ports:
    clk : in std_logic
    ena : in std_logic
    count : out std_logic_vector(WIDTH - 1 downto 0)

module led_driver
  ports:
    clk : in std_logic
    value : in std_logic_vector(7 downto 0)
    led : out std_logic
```

This project connects them. `hdlc new demo -l vhdl --example` writes the same
project and the two source files.

```json
{
  "version": 5,
  "top_name": "demo",
  "language": "Vhdl",
  "top_generics": [
    { "name": "CLK_DIV", "type_name": "integer", "default_value": "1000" }
  ],
  "top_ports": [
    { "name": "clk", "direction": "In", "port_type": "StdLogic", "bundle": null },
    { "name": "led", "direction": "Out", "port_type": "StdLogic", "bundle": null }
  ],
  "instances": [
    {
      "name": "u_pulse",
      "module_ref": "pulse_gen",
      "generic_map": { "DIV": "CLK_DIV" },
      "port_map": {
        "clk": { "TopPort": "clk" },
        "ena": { "Constant": "'1'" }
      },
      "position": [80, 80]
    },
    {
      "name": "u_led",
      "module_ref": "led_driver",
      "generic_map": {},
      "port_map": {
        "clk": { "TopPort": "clk" },
        "value": { "InstancePort": ["u_pulse", "count"] },
        "led": { "TopPort": "led" }
      },
      "position": [360, 80]
    }
  ],
  "aliases": { "u_pulse.count": "pulse_count" },
  "library_paths": ["pulse_gen.vhd", "led_driver.vhd"]
}
```

`u_pulse.count` has no `port_map` entry of its own. The reference from
`u_led.value` connects the two pins. One reference for each connection is
sufficient.

## Fields

Write each field that the example shows.

| Field | Content |
|---|---|
| `version` | The format version. Use the value from the example. |
| `top_name` | The name of the generated entity or module. |
| `language` | `"Vhdl"` or `"SystemVerilog"`. |
| `top_generics` | The generics of the generated module: `name`, `type_name`, and `default_value` (a string or `null`). |
| `top_ports` | The ports of the generated module: `name`, `direction` (`"In"`, `"Out"`, `"InOut"`), `port_type`, and `bundle` (use `null`). |
| `instances` | One entry for each instance. See below. |
| `aliases` | Signal names for nets. Use `{}` for none. |
| `library_paths` | The source files that define the leaf modules. A relative path starts at the directory of the `.hdlc` file. |

`port_type` has four forms:

| JSON value | VHDL | SystemVerilog |
|---|---|---|
| `"StdLogic"` | `std_logic` | `logic` |
| `{"StdLogicVector": {"high": {"Literal": 7}, "low": {"Literal": 0}, "dir": "Downto"}}` | `std_logic_vector(7 downto 0)` | `logic [7:0]` |
| `{"Record": "my_record_t"}` | the type name, written as given | the same |
| `{"Other": "unsigned(7 downto 0)"}` | the text, written as given | the same |

A bound can be an expression: `{"Expr": "WIDTH-1"}`. `dir` is `"Downto"` or
`"To"`.

Each instance has these fields:

| Field | Content |
|---|---|
| `name` | The instance label. It must be unique. |
| `module_ref` | The name of a module from `library_paths`. |
| `generic_map` | Generic name to value. Use `{}` to keep the defaults. |
| `port_map` | Port name to reference. The reference forms are in the format reference below. |
| `position` | `[x, y]`, the position of the block in the GUI. It has no effect on the generated HDL. |

## Rules

- Connect each input, or tie it to a `Constant`. An input with no connection
  is a warning, and VHDL rejects it unless the port has a default value.
- A net must have exactly one driver. The drivers are the instance outputs
  and the top-level inputs.
- An output that you do not use needs no `port_map` entry. `hdlc validate`
  gives a warning for it. This is the only warning that you can accept.
- An instance output can drive a slice of a wider net (`TopPortSlice`,
  `InstancePortSlice`). A reference to that output from a different instance
  connects to the same bits.
- To use a generated module as a leaf module of a different project, write it
  to a file with `hdlc codegen -o`, then put that file in the `library_paths`
  of the parent project.
