use app::App;

pub mod app;
pub mod board;
pub mod debug;
pub mod history;
pub mod moves;
pub mod popup;
pub mod seed_input;

fn main() {
    let terminal = ratatui::init();
    let _ = App::new().run(terminal);
    ratatui::restore();
}
