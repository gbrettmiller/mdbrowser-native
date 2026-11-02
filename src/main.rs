// Release builds on Windows are GUI apps: no console window opens beside them.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> iced::Result {
    mdbrowser::ui::app::run()
}
