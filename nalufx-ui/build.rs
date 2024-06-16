#[cfg(not(feature = "mcu-board-support"))]
fn main() {
    slint_build::compile("ui/desktop_window.slint").unwrap();
}