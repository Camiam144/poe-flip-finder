mod api;
mod app;
mod auth;
mod db;
mod logic;
mod models;

use app::App;

#[tokio::main]
async fn main() {
    let mut app = App::default();
    app.run().await;
}
