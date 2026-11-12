pub mod bridge;
pub mod editor;

use std::ffi::CString;
use std::path::Path;

unsafe extern "C" {
    fn run_gui(argc: *mut i32, argv: *mut *mut i8) -> i32;
}

/// Start the Qt event loop. `project` is the .hdlc file to open at startup.
///
/// The GUI receives exactly two kinds of argument: the program name and the
/// optional project path. clap has already consumed all other arguments.
pub fn run(project: Option<&Path>) -> i32 {
    let mut args: Vec<CString> = std::env::args()
        .take(1)
        .map(|a| CString::new(a).unwrap())
        .collect();
    if let Some(p) = project {
        // Absolute, so that the recent-projects list does not depend on the
        // directory the command ran in.
        let abs = std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf());
        args.push(CString::new(abs.to_string_lossy().as_bytes()).unwrap());
    }
    let mut argv: Vec<*mut i8> = args.iter().map(|a| a.as_ptr() as *mut i8).collect();
    let mut argc = argv.len() as i32;
    unsafe { run_gui(&mut argc, argv.as_mut_ptr()) }
}
