//! Helpers for the tests of the application.
//!
//! A test makes a [`Workspace`], starts the application in it, does the
//! actions of a user with the [`TestApp`], and reads the result in the window
//! and in the files of the workspace.
//!
//! The application of a test keeps its settings in the workspace. It does
//! not read or change the settings of the user.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

pub use headless_computer_use::testing::{Failure, Locator, TestApp, name, role};
use serde_json::Value;
use tempfile::TempDir;

/// The environment variable that gives the application a folder for its
/// settings.
const SETTINGS_DIR_ENV: &str = "HDLC_SETTINGS_DIR";

/// The name of the design that [`Workspace::with_example`] makes.
pub const EXAMPLE: &str = "demo";

/// The role of a block on the canvas. The name of a block is the name of
/// its instance.
pub const BLOCK: &str = "hdlc::InstanceItem";
/// The role of a pin of a block. The name of a pin is the name of its port.
pub const PIN: &str = "hdlc::PortPinItem";
/// The role of a top-level port on the canvas.
pub const TOP_PORT: &str = "hdlc::TopPortItem";
/// The role of a wire. See [`wire`] for its name.
pub const WIRE: &str = "hdlc::WireItem";

/// The block of the instance `instance`.
pub fn block(instance: &str) -> Locator {
    role(BLOCK).name(instance)
}

/// The pin `port` of the block of `instance`.
pub fn pin(instance: &str, port: &str) -> Locator {
    role(PIN).name(port).inside(block(instance))
}

/// The top-level port `port`.
pub fn top_port(port: &str) -> Locator {
    role(TOP_PORT).name(port)
}

/// The wire from `source` to `target`. The two are net keys: `top:<port>`
/// for a top-level port, `<instance>.<port>` for a pin.
pub fn wire(source: &str, target: &str) -> Locator {
    role(WIRE).name(&format!("{source} -> {target}"))
}

/// The entry `text` of the menu bar or of an open menu.
pub fn menu(text: &str) -> Locator {
    role("MenuItem").name(text)
}

/// The button `text`.
pub fn button(text: &str) -> Locator {
    role("Button").name(text)
}

/// The `hdlc` program of this source tree. The first call builds it, so that
/// a test never uses a stale program.
pub fn hdlc() -> &'static Path {
    static PROGRAM: OnceLock<PathBuf> = OnceLock::new();
    PROGRAM.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the package is in the source tree of hdlc");
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let build = Command::new(cargo)
            .args(["build", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .output()
            .expect("cargo starts");
        assert!(
            build.status.success(),
            "the build of hdlc failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let program = root.join("target/debug/hdlc");

        // A program that does not know the variable keeps its settings in
        // the settings of the user, and each test would change them.
        let bytes = fs::read(&program).expect("the built program");
        let variable = SETTINGS_DIR_ENV.as_bytes();
        assert!(
            bytes.windows(variable.len()).any(|part| part == variable),
            "{} does not read {SETTINGS_DIR_ENV}. The tests do not start it, because it would \
             change the settings of the user.",
            program.display()
        );
        program
    })
}

/// A temporary folder for the designs of one test, and a folder for the
/// settings of the application.
pub struct Workspace {
    dir: TempDir,
}

impl Workspace {
    /// A workspace with no design.
    pub fn empty() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("hdlc-ui-")
            .tempdir()
            .expect("a temporary folder");
        let workspace = Workspace { dir };
        fs::create_dir(workspace.designs()).expect("the folder of the designs");
        // Without this setting, a file dialog of the application starts in
        // the Documents folder of the user and shows its content.
        let settings = workspace.settings_file();
        fs::create_dir_all(settings.parent().expect("a folder")).expect("the settings folder");
        let start = format!(
            "[General]\ndefault_open_dir={}\n",
            workspace.designs().display()
        );
        fs::write(settings, start).expect("the settings file");
        workspace
    }

    /// A workspace with the example design of `hdlc new --example`: the
    /// design [`EXAMPLE`] with the blocks `u_pulse` and `u_led`, and their
    /// two HDL sources. The design has no warning.
    pub fn with_example() -> Self {
        let workspace = Self::empty();
        workspace.hdlc(&["new", "--language", "vhdl", "--example", EXAMPLE]);
        workspace
    }

    /// Runs the command-line program in the folder of the designs.
    pub fn hdlc(&self, arguments: &[&str]) {
        let output = Command::new(hdlc())
            .args(arguments)
            .current_dir(self.designs())
            .output()
            .expect("hdlc starts");
        assert!(
            output.status.success(),
            "hdlc {arguments:?} failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// The folder of the designs. The path has no symbolic link, so that it
    /// is equal to the path that the application shows.
    pub fn designs(&self) -> PathBuf {
        self.root().join("designs")
    }

    fn settings(&self) -> PathBuf {
        self.root().join("settings")
    }

    fn root(&self) -> PathBuf {
        self.dir
            .path()
            .canonicalize()
            .expect("the temporary folder")
    }

    /// The path of a file in the folder of the designs.
    pub fn file(&self, name: &str) -> PathBuf {
        self.designs().join(name)
    }

    /// The path of the design file of the design `name`.
    pub fn design(&self, name: &str) -> PathBuf {
        self.file(&format!("{name}.hdlc"))
    }

    /// The path of the recovery file of the design `name`.
    pub fn recovery_file(&self, name: &str) -> PathBuf {
        self.file(&format!(".{name}.hdlc.autosave"))
    }

    fn settings_file(&self) -> PathBuf {
        self.settings().join("hdl-compose/hdl-compose.ini")
    }

    /// The text of the settings file of the application.
    pub fn settings_text(&self) -> String {
        fs::read_to_string(self.settings_file()).expect("the settings file")
    }

    /// Starts the application with the design `name` of this workspace.
    pub fn open(&self, name: &str) -> Result<TestApp, Failure> {
        self.launch(Some(&self.design(name)))
    }

    /// Starts the application with no design argument. It then opens the
    /// design that the settings of this workspace have as the most recent.
    pub fn start(&self) -> Result<TestApp, Failure> {
        self.launch(None)
    }

    fn launch(&self, design: Option<&Path>) -> Result<TestApp, Failure> {
        let mut launcher = TestApp::builder([hdlc().to_string_lossy().into_owned()]);
        if let Some(design) = design {
            launcher = launcher.arg(design.to_string_lossy().into_owned());
        }
        launcher
            .env(
                SETTINGS_DIR_ENV,
                self.settings().to_string_lossy().into_owned(),
            )
            .launch()
    }

    /// The content of the design file of `name`.
    pub fn saved(&self, name: &str) -> Saved {
        let path = self.design(name);
        let text =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        Saved(
            serde_json::from_str(&text)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display())),
        )
    }
}

/// The content of a design file.
pub struct Saved(Value);

impl Saved {
    /// The names of the instances, in the order of the file.
    pub fn instances(&self) -> Vec<String> {
        self.instance_list()
            .iter()
            .map(|instance| instance["name"].as_str().expect("a name").to_string())
            .collect()
    }

    /// The position of the block of `instance`.
    pub fn position(&self, instance: &str) -> (f64, f64) {
        let position = &self.instance(instance)["position"];
        (
            position[0].as_f64().expect("x"),
            position[1].as_f64().expect("y"),
        )
    }

    /// What the port `port` of `instance` connects to, in the form of the
    /// design file. `Value::Null` is a port with no connection.
    pub fn connection(&self, instance: &str, port: &str) -> Value {
        self.instance(instance)["port_map"][port].clone()
    }

    fn instance(&self, name: &str) -> &Value {
        self.instance_list()
            .iter()
            .find(|instance| instance["name"] == name)
            .unwrap_or_else(|| panic!("the design file has no instance {name}"))
    }

    fn instance_list(&self) -> &Vec<Value> {
        self.0["instances"].as_array().expect("the instances")
    }
}

/// The window title of the application with the design `name` open.
pub fn title(name: &str) -> String {
    format!("HDL Compose \u{2014} {name}")
}

/// The window title while the design `name` has changes that are not saved.
pub fn title_not_saved(name: &str) -> String {
    format!("{} *", title(name))
}

/// Waits until `condition` is true. The expectations of [`TestApp`] wait for
/// the window. This function is for a result that is not in the window, for
/// example a file that the application writes later.
pub fn wait_for(what: &str, timeout: Duration, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + timeout;
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "No result after {timeout:?}: {what}."
        );
        thread::sleep(Duration::from_millis(50));
    }
}
