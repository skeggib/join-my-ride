// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod event_list;
mod event_publication;
mod page;
mod rest;

use seed::prelude::*;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        page: Page::Main(page::main::init(url, &mut orders.proxy(Msg::Main))),
    }
}

#[derive(Clone)]
enum Page {
    Main(page::main::Model),
}

struct Model {
    page: Page,
}

#[derive(Clone)]
enum Msg {
    Main(page::main::Msg),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Main(main_msg) => match &model.page {
            Page::Main(main_model) => {
                model.page = Page::Main(page::main::update(
                    main_msg,
                    main_model,
                    &mut orders.proxy(Msg::Main),
                ));
            }
        },
    }
}

fn view(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Main(main_model) => page::main::view(main_model).map_msg(Msg::Main),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
