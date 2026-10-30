use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_file = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo")).join("bindings.rs");

    let _skipped_members = windows_bindgen::bindgen([
        "--out",
        out_file.to_str().expect("OUT_DIR is valid UTF-8"),
        "--flat",
        "--implement",
        "--no-deps",
        "--no-allow",
        "--filter",
        "Windows.Win32.UI.Xaml.Diagnostics.IXamlDiagnostics",
        "Windows.Win32.UI.Xaml.Diagnostics.IVisualTreeService",
        "Windows.Win32.UI.Xaml.Diagnostics.IVisualTreeServiceCallback",
        "Windows.Win32.UI.Xaml.Diagnostics.IVisualTreeServiceCallback2",
        "Windows.Win32.UI.Xaml.Diagnostics.VisualElementState",
        "Windows.Win32.UI.Xaml.Diagnostics.ParentChildRelation",
        "Windows.Win32.UI.Xaml.Diagnostics.VisualElement",
        "Windows.Win32.UI.Xaml.Diagnostics.VisualMutationType",
        "Windows.UI.Xaml.IUIElement",
    ]);
}
