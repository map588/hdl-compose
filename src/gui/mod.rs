pub mod bridge;
pub mod editor;

use std::ffi::{CString, OsStr, c_char};
use std::path::Path;

unsafe extern "C" {
    fn run_gui(argc: *mut i32, argv: *mut *mut c_char, project: *const c_char) -> i32;
}

/// Start the Qt event loop. `project` is the .hdlc file to open at startup.
pub fn run(project: Option<&Path>) -> i32 {
    // Absolute, so that the recent-projects list does not depend on the
    // directory the command ran in.
    let project = project.map(|p| {
        let abs = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
        c_string(abs.as_os_str())
    });
    let args = qt_arguments(std::env::args_os());
    let mut argv: Vec<*mut c_char> = args.iter().map(|a| a.as_ptr() as *mut c_char).collect();
    let mut argc = argv.len() as i32;
    let project_ptr = project.as_ref().map_or(std::ptr::null(), |p| p.as_ptr());
    unsafe { run_gui(&mut argc, argv.as_mut_ptr(), project_ptr) }
}

/// The argument list for Qt: each argument of the process, with no change.
///
/// macOS sends each command-line argument that is not an option to the app
/// as a file to open. Qt drops such a request only when it finds the same
/// string in its own argument list. A list without the subcommand made the
/// word `gui` come back as a project to open, and each start showed
/// "No such file or directory (os error 2)".
fn qt_arguments(process_args: impl Iterator<Item = impl AsRef<OsStr>>) -> Vec<CString> {
    process_args.map(|a| c_string(a.as_ref())).collect()
}

fn c_string(text: &OsStr) -> CString {
    // An argument of the process or a path has no NUL byte.
    CString::new(text.as_encoded_bytes()).expect("no NUL byte in an argument")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: `hdlc gui` gave Qt only the program name, so macOS
    /// opened a project named `gui`.
    #[test]
    fn qt_gets_each_argument_of_the_process() {
        let args = qt_arguments(["hdlc", "gui", "demo.hdlc"].iter());
        let text: Vec<&str> = args.iter().map(|a| a.to_str().unwrap()).collect();
        assert_eq!(text, ["hdlc", "gui", "demo.hdlc"]);
    }
}
