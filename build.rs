use cxx_qt_build::CxxQtBuilder;

fn main() {
    // SAFETY: cc_builder callback only adds source files to the C++ build
    unsafe {
        CxxQtBuilder::new()
            .qt_module("Widgets")
            .qt_module("Gui")
            .file("src/gui/bridge.rs")
            .cc_builder(|cc| {
                cc.file("src/gui/app.cpp");
            })
            .build();
    }
}
