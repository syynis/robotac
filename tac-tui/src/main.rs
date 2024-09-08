use std::io;

use app::App;

pub mod app;

fn main() {
    let terminal = ratatui::init();
    let _ = App::new().run(terminal);
    ratatui::restore();
}
