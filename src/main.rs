mod api;
mod app;
mod logic;
mod models;

use app::App;

fn main() {
    let mut app = App::default();
    app.run();
}
