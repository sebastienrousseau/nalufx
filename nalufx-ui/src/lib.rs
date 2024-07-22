//! Main module for the application.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::ui::{HeaderAdapter, MainWindow, MenuOverviewAdapter, SettingsAdapter};
use slint::*;

/// This module contains the generated UI code for the application.
pub mod ui {
    slint::include_modules!();
}

/// Module containing various controllers for the application.
pub mod controllers {
    /// Submodule for header-related functionality.
    pub mod header;
}

/// Main function to initialize and run the application.
///
/// This function sets up the main window, initializes controllers,
/// and starts the kiosk mode timer if applicable.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // Provide better error messages in debug mode for WASM targets.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    // Create the main window
    let window = MainWindow::new().expect("Failed to create main window");

    // Set up the header controller
    let _header_timer = controllers::header::setup(&window);

    // Set up additional timer for specific feature combination
    #[cfg(all(not(feature = "mcu-board-support"), feature = "chrono"))]
    let _additional_timer = controllers::header::setup(&window);

    // Initialize kiosk mode timer
    let _kiosk_mode_timer = kiosk_timer(&window);

    // Run the main event loop
    window.run().expect("Failed to run main window");
}

/// Creates and starts a timer for kiosk mode functionality.
///
/// This function sets up a timer that periodically updates the current page
/// in kiosk mode, cycling through available pages.
///
/// # Arguments
///
/// * `window` - A reference to the MainWindow instance.
///
/// # Returns
///
/// Returns a Timer instance managing the kiosk mode updates.
fn kiosk_timer(window: &MainWindow) -> Timer {
    let kiosk_mode_timer = Timer::default();
    kiosk_mode_timer.start(TimerMode::Repeated, std::time::Duration::from_secs(4), {
        let window_weak = window.as_weak();
        move || {
            let window = window_weak.upgrade().expect("Window was dropped");
            let settings = SettingsAdapter::get(&window);
            let menu_overview = MenuOverviewAdapter::get(&window);

            if !settings.get_kiosk_mode_checked() {
                return;
            }

            let current_page = menu_overview.get_current_page();
            let count = menu_overview.get_count();

            let new_page = if current_page >= count - 1 {
                0
            } else {
                current_page + 1
            };

            menu_overview.set_current_page(new_page);
        }
    });
    kiosk_mode_timer
}
