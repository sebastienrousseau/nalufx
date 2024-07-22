use crate::HeaderAdapter;
use crate::MainWindow;
use dtt::DateTime;
use slint::ComponentHandle;
use slint::Timer;

/// Sets up a timer to update the date and time in the header of the application.
///
/// # Parameters
///
/// * `window` - A reference to the main window of the application.
///
/// # Returns
///
/// * A `Timer` instance that is started and configured to update the date and time.
///
pub fn setup(window: &MainWindow) -> Timer {
    // Create a new timer with default settings
    let update_timer = Timer::default();

    // Start the timer with a 300ms interval, repeating indefinitely
    update_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(300),
        {
            // Capture a weak reference to the main window
            let weak_window = window.as_weak();

            // Define the closure to be executed when the timer fires
            move || {
                // Attempt to upgrade the weak reference to a strong reference
                if let Some(window) = weak_window.upgrade() {
                    // Update the date and time in the header adapter
                    update(&window.global::<HeaderAdapter>());
                }
            }
        },
    );

    // Return the started timer
    update_timer
}

/// Updates the date and time in the header adapter.
///
/// # Parameters
///
/// * `header_adapter` - A reference to the header adapter where the date and time will be set.
///
/// # Returns
///
/// This function does not return any value.
fn update(header_adapter: &HeaderAdapter) {
    let dt = DateTime::new();
    let year = dt.year;
    let month = dt.month;
    let day = dt.day;
    let hour = dt.hour;
    let minute = dt.minute;

    // Convert 24-hour format to 12-hour format and determine AM/PM suffix
    let (formatted_hour, am_pm) = if hour == 0 {
        (12, "AM")
    } else if hour < 12 {
        (hour, "AM")
    } else if hour == 12 {
        (12, "PM")
    } else {
        (hour - 12, "PM")
    };

    let date = format!("{} {}, {}", month, day, year);
    let time = format!("{}:{:02}", formatted_hour, minute);

    header_adapter.set_date(slint::format!("{}", date));
    header_adapter.set_time(slint::format!("{}", time));
    header_adapter.set_time_suffix(slint::format!("{}", am_pm));
}
