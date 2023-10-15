// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::wasm_bindgen, App};
use frontend::app;

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", app::init, app::update, app::view);
}
