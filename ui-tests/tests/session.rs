//! Tests of what stays from one run of the application to the next: the
//! settings and the recovery file.

use std::process::Command;
use std::time::Duration;

use hdlc_ui_tests::{EXAMPLE, Failure, TestApp, Workspace, button, menu, role, title, wait_for};

/// The application writes the recovery file 12 s after the first change.
const RECOVERY_FILE_TIMEOUT: Duration = Duration::from_secs(20);
/// The settings domain of the application of the user.
const USER_SETTINGS: &str = "com.hdl-compose.hdl-compose";

/// Adds a block that is not saved, waits for the recovery file, and stops
/// the application as a crash does. Returns the workspace for the next run.
fn crash_with_a_change_that_is_not_saved() -> Result<Workspace, Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(role("ListItem").name("led_driver"))?;
    app.expect(role("TreeItem").name("led_driver_0 : led_driver"))
        .exists()?;
    assert!(!workspace.recovery_file(EXAMPLE).exists());
    wait_for("the recovery file", RECOVERY_FILE_TIMEOUT, || {
        workspace.recovery_file(EXAMPLE).exists()
    });
    kill(&mut app);
    Ok(workspace)
}

/// Stops the application with no time to save or to clean.
fn kill(app: &mut TestApp) {
    let pid = app.info().pid.to_string();
    let killed = Command::new("kill")
        .args(["-9", &pid])
        .status()
        .expect("`kill` runs");
    assert!(killed.success(), "kill -9 {pid}");
}

#[test]
fn recover_gives_the_changes_that_were_not_saved() -> Result<(), Failure> {
    let workspace = crash_with_a_change_that_is_not_saved()?;
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_pulse", "u_led"]);

    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("AlertMessage"))
        .name("Design \"demo\" was not closed correctly.")?;
    app.click(button("Recover"))?;

    app.expect(role("AlertMessage")).absent()?;
    app.expect(role("TreeItem").name("led_driver_0 : led_driver"))
        .exists()?;
    // The recovered changes are not in the design file until a save.
    assert_eq!(workspace.saved(EXAMPLE).instances(), ["u_pulse", "u_led"]);
    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;
    assert_eq!(
        workspace.saved(EXAMPLE).instances(),
        ["u_pulse", "u_led", "led_driver_0"]
    );
    assert!(!workspace.recovery_file(EXAMPLE).exists());
    Ok(())
}

#[test]
fn discard_removes_the_recovery_file_and_keeps_the_saved_design() -> Result<(), Failure> {
    let workspace = crash_with_a_change_that_is_not_saved()?;

    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("AlertMessage"))
        .name("Design \"demo\" was not closed correctly.")?;
    app.click(button("Discard"))?;

    app.expect(role("AlertMessage")).absent()?;
    app.expect(role("TreeItem").name("u_led : led_driver"))
        .exists()?;
    app.expect(role("TreeItem").name("led_driver_0 : led_driver"))
        .absent()?;
    assert!(!workspace.recovery_file(EXAMPLE).exists());
    drop(app);

    // The next run has no question.
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("TreeItem").name("u_led : led_driver"))
        .exists()?;
    app.expect(role("AlertMessage")).absent()?;
    Ok(())
}

#[test]
fn a_save_removes_the_recovery_file() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.double_click(role("ListItem").name("led_driver"))?;
    wait_for("the recovery file", RECOVERY_FILE_TIMEOUT, || {
        workspace.recovery_file(EXAMPLE).exists()
    });

    app.click(button("Save"))?;
    app.expect(role("StatusBar")).name("Saved")?;

    assert!(!workspace.recovery_file(EXAMPLE).exists());
    Ok(())
}

/// The tests must not read or change the settings of the user. The recent
/// designs are a setting that each open of a design writes.
#[test]
fn the_settings_of_a_test_are_not_the_settings_of_the_user() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let design = workspace.design(EXAMPLE);
    let design = design.to_string_lossy();
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;

    wait_for(
        "the recent design in the settings of the test",
        Duration::from_secs(5),
        || {
            workspace
                .settings_text()
                .contains(&format!("recent_projects={design}"))
        },
    );
    let user = Command::new("defaults")
        .args(["export", USER_SETTINGS, "-"])
        .output()
        .expect("`defaults` runs");
    let user = String::from_utf8_lossy(&user.stdout);
    assert!(
        !user.contains(&*design),
        "the settings of the user have {design}"
    );
    Ok(())
}

#[test]
fn a_start_with_no_design_opens_the_most_recent_design() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;
    drop(app);

    let mut app = workspace.start()?;

    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(role("TreeItem").name("u_led : led_driver"))
        .exists()?;
    Ok(())
}

#[test]
fn the_recent_menu_opens_a_design_that_was_open_before() -> Result<(), Failure> {
    let workspace = Workspace::with_example();
    workspace.hdlc(&["new", "--language", "vhdl", "amp"]);
    let mut app = workspace.open(EXAMPLE)?;
    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.double_click(role("ListItem").name("amp (not generated)"))?;
    app.expect(role("Window")).name(&title("amp"))?;

    app.click(menu("File"))?;
    app.click(menu("Open Recent"))?;
    app.click(menu(&workspace.design(EXAMPLE).to_string_lossy()))?;

    app.expect(role("Window")).name(&title(EXAMPLE))?;
    app.expect(role("TreeItem").name("u_led : led_driver"))
        .exists()?;
    Ok(())
}
