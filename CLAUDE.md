USER PROMPT START

# Project Intention

This project is to design a simple block editor for VHDL/Verilog/SystemVerilog, to generate a structural module/entity which uses intermediate signal to generate the mapping between the submodules.

Its primary goal is to be a simple, easy to use, and easy to extend tool for synthesizable code.  

The output of each "project" is a higher level structural module/entity file, which can either act as the top level module, or be used as a sub-module.


# GUIDLINES

## General

Complexity has a cost
- Don't overcomplicate things
- Don't overengineer things
- Don't overdesign things
- Don't overspec things
- Don't overdetermine things
- Don't overoptimize things
- Don't overcommunicate things

Every line of code is a tradeoff, it adds maintainence burden and another thing that could go wrong.
Use abstractions wisely.

## Tool Usage

### tokensave
- Tokensave is a tool for navigating code, try to use it in place of Grep/Read commands.  It indexes your codebase, and provides a queryable interface to it.


### rust-analyzer 
- rust-analyzer provides rust code intelligence, try to use it before running Cargo commands for checking rust correctness



### memstate
- Intelligent memory management backed by SQLite, to record anything you want to remember, use it to record design discussions, architecture decisions, progress, status, etc.
    - Each keypath is a unique identifier, and can be used to retrieve the value at that keypath, re-writing the keypath to the same value will not delete the old value,
      it behaves like version control.

    #### **Use to store information about this project.  Any "note to self" you might find useful to a new agent without your current context**

    #### **ALWAYS** check for entries in your memory on entry, and **ALWAYS** document your progress (technical, architectural, contextual, etc) to your memory to help out your future self.

    #### Memstate project_id for this codebase
    - Historical: `block_editor` (34+ memories — design decisions, sessions, gotchas, TODOs from when this was a non-git working copy at `~/git/hdl_tooling/block-editor`)
    - Repo-side: `hdl_compose` and `hdl_compose_dev` hold pointers back to `block_editor`
    - **On entry, load `block_editor` first.** Write new memories to `block_editor` to preserve continuity.

USER PROMPT END

Please follow the guidelines within the USER GUIDELINES START and END, they were hand-written by the actual human you are meant to help.  I don't know how many prompts you get automatically by all the
LLM scaffolding, but these instructions are from me, and I would greatly appreciate if you did your best to follow them.

Thank you for your help!
