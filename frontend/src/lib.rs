// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod event_list;
mod event_publication;
mod page;
mod rest;

use seed::{prelude::*, *};
use std::str::FromStr;

fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
        page: Page::Main(page::main::init(url, &mut orders.proxy(Msg::Main))),
    }
}

fn parse_url(mut url: Url) -> Option<Route> {
    match url.next_path_part() {
        Some("event") => Some(Route::Event(
            common::Id::from_str(url.next_path_part()?).ok()?,
        )),
        _ => None,
    }
}

#[derive(Clone)]
enum Page {
    Main(page::main::Model),
    Event(page::event::Model),
}

struct Model {
    page: Page,
}

#[derive(Clone)]
enum Msg {
    UrlChanged(subs::UrlChanged),
    Main(page::main::Msg),
    Event(page::event::Msg),
}

#[derive(Clone)]
enum Route {
    Main,
    Event(common::Id),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(url) => {
            model.page = match parse_url(url.0.clone()) {
                Some(Route::Main) => Page::Main(page::main::init(
                    url.0.clone(),
                    &mut orders.proxy(Msg::Main),
                )),
                Some(Route::Event(id)) => {
                    Page::Event(page::event::init(url.0, &mut orders.proxy(Msg::Event)))
                }
                None => Page::Main(page::main::init(url.0, &mut orders.proxy(Msg::Main))),
            };
        }
        Msg::Main(main_msg) => {
            if let Page::Main(model) = &mut model.page {
                *model = page::main::update(main_msg, model, &mut orders.proxy(Msg::Main));
            }
        }
        Msg::Event(event_msg) => {
            if let Page::Event(model) = &mut model.page {
                *model = page::event::update(event_msg, model, &mut orders.proxy(Msg::Event));
            }
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Main(model) => page::main::view(model).map_msg(Msg::Main),
        Page::Event(model) => page::event::view(model).map_msg(Msg::Event),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
