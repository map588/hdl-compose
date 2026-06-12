use cxx_qt_build::CxxQtBuilder;

fn main() {
    // SAFETY: cc_builder callback only adds source files to the C++ build.
    // Note: arm64-macOS workaround for Qt's qyieldcpu.h lives in
    // .cargo/config.toml (`CXXFLAGS_aarch64_apple_darwin`) so it reaches
    // every cc invocation including cxx-qt's own build script.
    unsafe {
        CxxQtBuilder::new()
            .qt_module("Widgets")
            .qt_module("Gui")
            .file("src/gui/bridge.rs")
            .cc_builder(|cc| {
                cc.file("src/gui/app.cpp");
                cc.file("src/gui/canvas.cpp");
                cc.file("src/gui/items.cpp");
            })
            .build();
    }
}
