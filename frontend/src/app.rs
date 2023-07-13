use crate::pages;
use seed::prelude::*;

pub fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
        page: page_from_url(&mut url, orders),
    }
}

fn parse_url(url: &mut Url) -> Route {
    match url.next_path_part() {
        Some("event") => Route::Event,
        _ => Route::Main,
    }
}

fn page_from_url(url: &mut Url, orders: &mut impl Orders<Msg>) -> Page {
    match parse_url(url) {
        Route::Main => Page::Main(pages::main::init(url, &mut orders.proxy(Msg::Main))),
        Route::Event => Page::Event(pages::event::init(url, &mut orders.proxy(Msg::Event))),
    }
}

#[derive(Clone)]
enum Page {
    Main(pages::main::Model),
    Event(pages::event::Model),
}

pub struct Model {
    page: Page,
}

#[derive(Clone)]
pub enum Msg {
    UrlChanged(subs::UrlChanged),
    Main(pages::main::Msg),
    Event(pages::event::Msg),
}

#[derive(Clone)]
enum Route {
    Main,
    Event,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(url_changed) => {
            let mut url = url_changed.0;
            model.page = page_from_url(&mut url, orders);
        }
        Msg::Main(main_msg) => {
            if let Page::Main(model) = &mut model.page {
                pages::main::update(main_msg, model, &mut orders.proxy(Msg::Main));
            }
        }
        Msg::Event(event_msg) => {
            if let Page::Event(model) = &mut model.page {
                pages::event::update(event_msg, model, &mut orders.proxy(Msg::Event));
            }
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Main(model) => pages::main::view(model).map_msg(Msg::Main),
        Page::Event(model) => pages::event::view(model).map_msg(Msg::Event),
    }
}
