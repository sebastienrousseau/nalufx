#[cfg(not(feature = "mcu-board-support"))]
fn main() {
    slint_build::compile("ui/window.slint").unwrap();
}