// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(windows, any(feature = "portable", test)))]
mod portable;

fn main() {
    #[cfg(all(windows, feature = "portable"))]
    {
        match portable::prepare_embedded() {
            Ok(runtime) => {
                // Set before Tauri or any of its worker threads are started.
                std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", runtime);
            }
            Err(error) => {
                portable::show_error(&error);
                std::process::exit(1);
            }
        }
    }
    silo_lib::run()
}
