mod game;
mod ui;
use std::io;

fn main() -> Result<(), io::Error> {
    // Configure log
    tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    ui::run_app()?;

    Ok(())
}
