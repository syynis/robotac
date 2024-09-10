use std::io;

use app::App;

pub mod app;
pub mod board;
pub mod moves;

fn main() {
    let terminal = ratatui::init();
    let _ = App::new().run(terminal);
    ratatui::restore();
}
