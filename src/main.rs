mod ui;
mod download_systems;

use download_systems::download_systems_if_needed;

#[tokio::main]
async fn main() -> iced::Result {
    let args = std::env::args().collect::<Vec<_>>();
    download_systems_if_needed(args).await.unwrap();

    ui::run()
}