#![no_std]
#![cfg_attr(all(feature = "mcu-board-support", not(feature = "simulator")), no_main)]

use nalufx_ui_lib::main as lib_main;

/// This function is the entry point of the application when the `mcu-board-support` feature is not enabled.
/// It calls the `lib_main` function from the `nalufx_ui_lib` crate.
///
/// # Arguments
///
/// None.
///
/// # Returns
///
/// None.
///
/// # Panics
///
/// This function does not panic.
#[cfg(not(feature = "mcu-board-support"))]
pub fn main() {
    lib_main();
}

/// This is the main function for the application when the `mcu-board-support` feature is enabled.
/// It is marked with the `#[mcu_board_support::entry]` attribute to indicate that it is the entry point of the application.
///
/// # Arguments
///
/// None.
///
/// # Returns
///
/// This function does not return a value, but it is marked with the `!` type to indicate that it will never return.
///
/// # Panics
///
/// This function will panic if the `lib_main` function panics.
/// It will also panic with the message "The MCU demo should not quit" if it reaches this point.
#[cfg(feature = "mcu-board-support")]
#[mcu_board_support::entry]
fn main() -> ! {
    /// Initializes the MCU board support.
    mcu_board_support::init();

    /// Calls the main function from the `nalufx_ui_lib` crate.
    lib_main();

    /// Panics with a message indicating that the MCU demo should not quit.
    panic!("The MCU demo should not quit")
}
