//! Tests of the work with a design: open, new, add, save, generate, and the
//! question about changes that are not saved.

use std::fs;

use hdlc_ui_tests::{
    EXAMPLE, Failure, Workspace, button, menu, name, role, title, title_not_saved,
};

/// The entry of a list, for example a module of the library.
fn list_item(text: &str) -> hdlc_ui_tests::Locator {
    role("ListItem").name(text)
}

/// The entry of the tree of the design: `<instance> : <module>`.
fn tree_item(text: &str) -> hdlc_ui_tests::Locator {
    role("TreeItem").name(text)
}

#[test]
fn a_design_opens_with_its_blocks_and_its_modules() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(tree_item("u_pulse : pulse_gen")).exists()?;
    app.expect(tree_item("u_led : led_driver")).exists()?;
    app.expect(list_item("pulse_gen")).exists()?;
    app.expect(list_item("led_driver")).exists()?;
    app.expect(list_item("demo (not generated)")).exists()?;
    app.expect(button("No issues")).exists()?;
    Ok(())
}

/// With no design argument and no recent design, the window is empty. The
/// dialog starts in the folder that the settings give.
#[test]
fn the_open_dialog_opens_the_chosen_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.start()?;
    app.expect(role("Window")).name("HDL Compose")?;

    app.click(button("Open"))?;
    app.expect(role("Dialog")).name("Open Design")?;
    app.double_click(tree_item("demo.hdlc"))?;

    app.expect(role("Dialog")).absent()?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(tree_item("u_led : led_driver")).exists()?;
    Ok(())
}

#[test]
fn generate_writes_the_hdl_next_to_the_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let generated = workspace.file("demo.vhd");
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(list_item("demo (not generated)")).exists()?;
    assert!(!generated.exists());

    app.click(button("Generate HDL"))?;

    app.expect(role("StatusBar"))
        .name(&format!("Generated {}", generated.display()))?;
    app.expect(list_item("demo")).exists()?;
    let text = fs::read_to_string(&generated).expect("the generated file");
    assert!(text.contains("entity demo is"), "{text}");
    for instance in ["u_pulse", "u_led"] {
        assert!(text.contains(instance), "no {instance} in:\n{text}");
    }
    Ok(())
}

#[test]
fn a_double_click_on_a_module_adds_a_block_and_save_writes_it() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.double_click(list_item("led_driver"))?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .exists()?;
    // The new block has inputs with no connection.
    app.expect(button("No issues")).absent()?;
    // The design file changes only with a save.
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_pulse", "u_led"]);

    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    assert_eq!(
        workspace.saved(EXAMPLE).instances(),
        ["u_pulse", "u_led", "led_driver_0"]
    );
    Ok(())
}

/// The title has the mark ` *` while the design has changes that are not
/// saved. Undo back to the saved design removes the mark.
#[test]
fn the_title_marks_changes_that_are_not_saved() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;

    app.double_click(list_item("led_driver"))?;
    app.expect(role("Window")).name(&title_not_saved(EXAMPLE))?;
    app.click(button("Save"))?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;

    app.double_click(list_item("pulse_gen"))?;
    app.expect(role("Window")).name(&title_not_saved(EXAMPLE))?;
    app.click(menu("Edit"))?;
    app.click(menu("Undo"))?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;
    Ok(())
}

#[test]
fn undo_removes_the_added_block_and_redo_adds_it_again() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(list_item("pulse_gen"))?;
    app.expect(tree_item("pulse_gen_0 : pulse_gen")).exists()?;

    app.click(menu("Edit"))?;
    app.click(menu("Undo"))?;
    app.expect(tree_item("pulse_gen_0 : pulse_gen")).absent()?;

    app.click(menu("Edit"))?;
    app.click(menu("Redo"))?;
    app.expect(tree_item("pulse_gen_0 : pulse_gen")).exists()?;
    Ok(())
}

/// With a design open, a new design goes into the folder of that design,
/// and no dialog asks for a place.
#[test]
fn a_new_design_goes_into_the_folder_of_the_open_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.click(button("New"))?;
    app.expect(role("Dialog")).name("New Design")?;
    app.type_text("amp")?;
    app.press("Return")?;

    app.expect(role("Window")).name(&title("amp"))?;
    app.expect(list_item("amp (not generated)")).exists()?;
    app.expect(list_item("demo (not generated)")).exists()?;
    assert_eq!(workspace.saved("amp").instances(), Vec::<String>::new());
    Ok(())
}

#[test]
fn a_double_click_in_the_list_of_designs_opens_that_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    workspace.hdlc(&["new", "--language", "vhdl", "amp"]);
    let mut app = workspace.open("amp")?;
    app.expect(role("Window")).name(&title("amp"))?;
    app.expect(tree_item("u_pulse : pulse_gen")).absent()?;

    app.double_click(list_item("demo (not generated)"))?;

    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(tree_item("u_pulse : pulse_gen")).exists()?;
    Ok(())
}

/// The three answers to the question about changes that are not saved.
#[test]
fn cancel_keeps_the_design_and_its_changes() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(list_item("led_driver"))?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .exists()?;

    app.click(button("New"))?;
    app.expect(role("AlertMessage"))
        .name("Design \"demo\" has unsaved changes.")?;
    app.click(button("Cancel").inside(role("AlertMessage")))?;

    app.expect(role("AlertMessage")).absent()?;
    app.expect(role("Dialog")).absent()?;
    app.expect(role("Window")).name(&title_not_saved(EXAMPLE))?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .exists()?;
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_pulse", "u_led"]);
    Ok(())
}

#[test]
fn save_in_the_question_writes_the_changes_before_the_new_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(list_item("led_driver"))?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .exists()?;

    app.click(button("New"))?;
    app.click(button("Save").inside(role("AlertMessage")))?;

    app.expect(role("Dialog")).name("New Design")?;
    assert_eq!(
        workspace.saved(EXAMPLE).instances(),
        ["u_pulse", "u_led", "led_driver_0"]
    );
    Ok(())
}

#[test]
fn discard_in_the_question_drops_the_changes() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(list_item("led_driver"))?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .exists()?;

    app.click(button("New"))?;
    app.click(button("Discard").inside(role("AlertMessage")))?;
    app.expect(role("Dialog")).name("New Design")?;
    app.type_text("amp")?;
    app.press("Return")?;
    app.expect(role("Window")).name(&title("amp"))?;

    // The design on disk and the design that opens again have no new block.
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_pulse", "u_led"]);
    app.double_click(list_item("demo (not generated)"))?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(tree_item("u_led : led_driver")).exists()?;
    app.expect(tree_item("led_driver_0 : led_driver"))
        .absent()?;
    app.expect(role("AlertMessage")).absent()?;
    Ok(())
}

#[test]
fn rename_in_the_edit_menu_renames_the_selected_block() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.click(tree_item("u_led : led_driver"))?;
    app.click(menu("Edit"))?;
    app.click(menu("Rename Instance..."))?;
    app.expect(role("Dialog")).name("Rename Instance")?;
    app.expect(role("EditableText")).value("u_led")?;
    app.press("Ctrl+A")?;
    app.type_text("u_lamp")?;
    app.press("Return")?;

    app.expect(tree_item("u_lamp : led_driver")).exists()?;
    app.expect(tree_item("u_led : led_driver")).absent()?;
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    let saved = workspace.saved(EXAMPLE);
    assert_eq!(saved.instances(), ["u_pulse", "u_lamp"]);
    // The wire from the other block stays on the renamed block.
    assert_eq!(
        saved.connection("u_lamp", "value"),
        serde_json::json!({ "InstancePort": ["u_pulse", "count"] })
    );
    Ok(())
}

#[test]
fn delete_in_the_edit_menu_removes_the_selected_block() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;

    app.click(tree_item("u_pulse : pulse_gen"))?;
    app.click(menu("Edit"))?;
    app.click(menu("Delete Instance"))?;

    app.expect(tree_item("u_pulse : pulse_gen")).absent()?;
    // The input of the other block has no source now.
    app.expect(button("No issues")).absent()?;
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    let saved = workspace.saved(EXAMPLE);
    assert_eq!(saved.instances(), ["u_led"]);
    assert_eq!(saved.connection("u_led", "value"), serde_json::Value::Null);
    Ok(())
}

/// With no selected block, the two actions say so and change nothing.
#[test]
fn rename_and_delete_with_no_selected_block_change_nothing() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(tree_item("u_led : led_driver")).exists()?;

    app.click(menu("Edit"))?;
    app.click(menu("Delete Instance"))?;
    app.expect(role("StatusBar"))
        .name("Delete: select an instance first")?;
    app.click(menu("Edit"))?;
    app.click(menu("Rename Instance..."))?;
    app.expect(role("StatusBar"))
        .name("Rename: select an instance first")?;

    app.expect(role("Dialog")).absent()?;
    app.expect(name("u_pulse : pulse_gen")).exists()?;
    app.expect(name("u_led : led_driver")).exists()?;
    Ok(())
}
