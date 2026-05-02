pub mod bridge;

use std::ffi::CString;

unsafe extern "C" {
    fn run_gui(argc: *mut i32, argv: *mut *mut i8) -> i32;
}

pub fn run() -> i32 {
    let args: Vec<CString> = std::env::args()
        .map(|a| CString::new(a).unwrap())
        .collect();
    let mut argv: Vec<*mut i8> = args.iter().map(|a| a.as_ptr() as *mut i8).collect();
    let mut argc = argv.len() as i32;
    unsafe { run_gui(&mut argc, argv.as_mut_ptr()) }
}
