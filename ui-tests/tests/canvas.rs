//! Tests of the canvas: the blocks, the pins and the wires of the design.
//!
//! A block, a pin and a wire are elements with a name, see the functions
//! `block`, `pin`, `top_port` and `wire` of the helpers.

use hdlc_ui_tests::{
    EXAMPLE, Failure, WIRE, Workspace, block, button, menu, pin, role, top_port, wire,
};
use serde_json::{Value, json};

/// The largest difference in pixels between two places that are the same
/// place. The view draws at a zoom, thus a place is not exact.
const SAME_PLACE_PX: i32 = 4;

#[test]
fn the_canvas_shows_each_block_pin_and_wire_of_the_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.expect(block("u_pulse")).exists()?;
    app.expect(block("u_led")).exists()?;
    for (instance, port) in [
        ("u_pulse", "clk"),
        ("u_pulse", "ena"),
        ("u_pulse", "count"),
        ("u_led", "clk"),
        ("u_led", "value"),
        ("u_led", "led"),
    ] {
        app.expect(pin(instance, port)).exists()?;
    }
    app.expect(top_port("clk")).exists()?;
    app.expect(top_port("led")).exists()?;
    app.expect(wire("top:clk", "u_pulse.clk")).exists()?;
    app.expect(wire("top:clk", "u_led.clk")).exists()?;
    app.expect(wire("u_pulse.count", "u_led.value")).exists()?;
    app.expect(wire("top:led", "u_led.led")).exists()?;
    assert_eq!(app.find_all(role(WIRE))?.len(), 4);
    Ok(())
}

/// The two blocks of the example are in adjacent columns. A drag of one
/// column width puts the block in the next column.
#[test]
fn a_drag_moves_a_block_to_the_next_column_and_undo_puts_it_back() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    let pulse = app.find(block("u_pulse"))?;
    let start = app.find(block("u_led"))?;
    let column = start.x - pulse.x;
    assert!(column > 50, "the blocks are not in two columns: {column}");
    let saved_x = workspace.saved(EXAMPLE).position("u_led").0;

    let from = start.click_point();
    // The release is not exactly on the column. The block goes to the
    // column that is nearest.
    app.drag(block("u_led"), (from.x + column - column / 5, from.y))?;

    let moved = app.find(block("u_led"))?;
    assert!(
        (moved.x - (start.x + column)).abs() <= SAME_PLACE_PX,
        "the block is at x {}, one column from {} is {}",
        moved.x,
        start.x,
        start.x + column
    );
    assert_eq!(app.find(block("u_pulse"))?.x, pulse.x);
    app.expect(wire("u_pulse.count", "u_led.value")).exists()?;
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    let moved_x = workspace.saved(EXAMPLE).position("u_led").0;
    assert!(moved_x > saved_x, "the design file has x {moved_x}");

    app.click(menu("Edit"))?;
    app.click(menu("Undo"))?;
    app.wait_until("the block u_led is in its first column again", |snapshot| {
        snapshot
            .outline()
            .lines()
            .any(|line| line.contains("\"u_led\"") && line.contains(&format!("[{},", start.x)))
    })?;
    Ok(())
}

/// A press and a release at one place is a click: the block does not move.
#[test]
fn a_click_on_a_block_selects_it_and_does_not_move_it() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    let start = app.find(block("u_led"))?;

    app.click(block("u_led"))?;

    app.expect(block("u_led")).state("selected")?;
    app.expect(block("u_pulse")).no_state("selected")?;
    app.expect(role("TreeItem").name("u_led : led_driver"))
        .state("selected")?;
    assert_eq!(app.find(block("u_led"))?.x, start.x);
    // No change: the question about changes that are not saved does not come.
    app.click(button("New"))?;
    app.expect(role("Dialog")).name("New Design")?;
    Ok(())
}

#[test]
fn a_click_on_two_pins_makes_a_wire() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(role("ListItem").name("led_driver"))?;
    app.expect(block("led_driver_0")).exists()?;

    // From a top-level port to an input.
    app.click(top_port("clk"))?;
    app.click(pin("led_driver_0", "clk"))?;
    app.expect(wire("top:clk", "led_driver_0.clk")).exists()?;
    // From an output that has a load already to a second input.
    app.click(pin("u_pulse", "count"))?;
    app.click(pin("led_driver_0", "value"))?;
    app.expect(wire("u_pulse.count", "led_driver_0.value"))
        .exists()?;

    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    let saved = workspace.saved(EXAMPLE);
    assert_eq!(
        saved.connection("led_driver_0", "clk"),
        json!({ "TopPort": "clk" })
    );
    assert_eq!(
        saved.connection("led_driver_0", "value"),
        json!({ "InstancePort": ["u_pulse", "count"] })
    );
    // The first load of the output keeps its wire.
    assert_eq!(
        saved.connection("u_led", "value"),
        json!({ "InstancePort": ["u_pulse", "count"] })
    );
    Ok(())
}

/// A vector of 8 bits and a port of 1 bit do not connect.
#[test]
fn two_pins_with_different_widths_make_no_wire() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(role("ListItem").name("led_driver"))?;
    app.expect(block("led_driver_0")).exists()?;

    app.click(top_port("clk"))?;
    app.click(pin("led_driver_0", "value"))?;
    // A wire that is correct shows that the tool is ready again, and that
    // the wrong wire did not come in the time between.
    app.click(top_port("clk"))?;
    app.click(pin("led_driver_0", "clk"))?;
    app.expect(wire("top:clk", "led_driver_0.clk")).exists()?;

    app.expect(wire("top:clk", "led_driver_0.value")).absent()?;
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    assert_eq!(
        workspace.saved(EXAMPLE).connection("led_driver_0", "value"),
        Value::Null
    );
    Ok(())
}

#[test]
fn the_delete_key_removes_the_selected_wire() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.click(wire("u_pulse.count", "u_led.value"))?;
    app.expect(wire("u_pulse.count", "u_led.value"))
        .state("selected")?;
    app.press("Delete")?;

    app.expect(wire("u_pulse.count", "u_led.value")).absent()?;
    assert_eq!(app.find_all(role(WIRE))?.len(), 3);
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    let saved = workspace.saved(EXAMPLE);
    assert_eq!(saved.connection("u_led", "value"), Value::Null);
    assert_eq!(saved.instances(), ["u_pulse", "u_led"]);
    Ok(())
}

#[test]
fn the_delete_key_removes_the_selected_block_with_its_wires() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.click(block("u_pulse"))?;
    app.expect(block("u_pulse")).state("selected")?;
    app.press("Delete")?;

    app.expect(block("u_pulse")).absent()?;
    app.expect(wire("top:clk", "u_pulse.clk")).absent()?;
    app.expect(wire("u_pulse.count", "u_led.value")).absent()?;
    app.expect(role("TreeItem").name("u_pulse : pulse_gen"))
        .absent()?;
    app.expect(block("u_led")).exists()?;
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_led"]);
    Ok(())
}
