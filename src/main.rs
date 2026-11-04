// Release builds on Windows are GUI apps: no console window opens beside them, so `--version`
// prints nothing there and only its exit code can be checked.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mdbrowser::services::files;

fn main() -> iced::Result {
    if files::asks_for_version(std::env::args()) {
        println!("mdbrowser {}", mdbrowser::session::about::version());
        return Ok(());
    }
    mdbrowser::ui::app::run()
}
