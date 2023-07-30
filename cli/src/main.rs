use common::Game;
use log::info;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .env()
        .without_timestamps()
        .with_colors(true)
        .init()
        .unwrap();

    println!("Care for a round of Flin?");

    let mut g = Game::new();
    g.play(true);
    if let Some(winner) = g.winner {
        info!("The winner is: {}", winner);
    } else {
        info!("Draw!");
    }
}
