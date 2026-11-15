// The C++ side of the GUI. A build with no `gui` feature has no C++ and
// needs no Qt.
#[cfg(feature = "gui")]
fn main() {
    use cxx_qt_build::CxxQtBuilder;

    let builder = CxxQtBuilder::new()
        .qt_module("Widgets")
        .qt_module("Gui")
        .file("src/gui/bridge.rs");
    // The tests of the C++ side: a QObject with the test functions (moc
    // reads the header) and the Qt Test module.
    #[cfg(feature = "gui-tests")]
    let builder = builder.qt_module("Test").cpp_file("src/gui/gui_tests.h");

    // SAFETY: cc_builder callback only adds source files to the C++ build.
    // Note: arm64-macOS workaround for Qt's qyieldcpu.h lives in
    // .cargo/config.toml (`CXXFLAGS_aarch64_apple_darwin`) so it reaches
    // every cc invocation including cxx-qt's own build script.
    unsafe {
        builder
            .cc_builder(|cc| {
                cc.file("src/gui/app.cpp");
                cc.file("src/gui/canvas.cpp");
                cc.file("src/gui/items.cpp");
                #[cfg(feature = "gui-tests")]
                {
                    cc.file("src/gui/gui_tests.cpp");
                    cc.define("HDLC_GUI_TESTS", None);
                }
            })
            .build();
    }
}

#[cfg(not(feature = "gui"))]
fn main() {}
