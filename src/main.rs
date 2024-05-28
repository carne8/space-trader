mod ui;
mod download_systems;

use iced::Application;
use download_systems::download_systems;


#[tokio::main]
async fn main() -> iced::Result {
    let mut args = std::env::args();
    let should_download_systems = args.find(|arg| arg == "--download-systems").is_some();
    if should_download_systems {
        download_systems().await.unwrap();
    }

    ui::App::run(iced::Settings {
        antialiasing: true,
        ..iced::Settings::default()
    })
}