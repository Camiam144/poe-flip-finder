mod api;
mod app;
mod auth;
mod logic;
mod models;

use app::App;

#[tokio::main]
async fn main() {
    let mut app = App::default();
    app.run().await;
}
