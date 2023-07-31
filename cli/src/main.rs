use common::Game;
use log::{debug, error, info, trace, warn};
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

    let mut g = Game::new(true, Some(notify));
    g.play();
    if let Some(winner) = g.winner {
        info!("The winner is: {}", winner);
    } else {
        info!("Draw!");
    }
}

fn notify(msg: &str, level: log::Level) {
    match level {
        log::Level::Error => error!("{}", msg),
        log::Level::Warn => warn!("{}", msg),
        log::Level::Info => info!("{}", msg),
        log::Level::Debug => debug!("{}", msg),
        log::Level::Trace => trace!("{}", msg),
    }
}
