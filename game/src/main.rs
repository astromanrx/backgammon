// https://www.bkgm.com/rules.html
mod utils;
mod gdk;
mod drawing;
mod ui;
mod resources;
mod states;
mod components;
mod game;

use game::run_game;


#[tokio::main]
async fn main() {    
    run_game().await;
}