use crate::{orders::perform_cmd, pages};
use seed::{
    prelude::{subs::UrlChanged, *},
    *,
};

pub fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    let context = Context { username: None };
    let current_url = url.clone();
    Model {
        page: page_from_url(&mut url, None, &context, orders),
        current_url: current_url,
        context: context,
    }
}

fn parse_url(url: &mut Url) -> Route {
    match url.next_path_part() {
        Some("event") => Route::Event,
        Some("login") => Route::Login,
        _ => Route::Main,
    }
}

fn page_from_url(
    url: &mut Url,
    previous_url: Option<Url>,
    context: &Context,
    orders: &mut impl Orders<Msg>,
) -> Page {
    match parse_url(url) {
        Route::Main => Page::Main(pages::main::init(url, &mut orders.proxy(Msg::Main))),
        Route::Event => Page::Event(pages::event::init(url, &mut orders.proxy(Msg::Event))),
        Route::Login => Page::Login(pages::login::init(
            url,
            previous_url,
            context,
            &mut orders.proxy(Msg::Login),
        )),
    }
}

enum Page {
    Main(pages::main::Model),
    Event(pages::event::Model),
    Login(pages::login::Model),
}

pub struct Context {
    pub username: Option<String>,
}

pub struct Model {
    page: Page,
    current_url: Url,
    context: Context,
}

pub enum Msg {
    UrlChanged(subs::UrlChanged),
    Main(pages::main::Msg),
    Event(pages::event::Msg),
    Login(pages::login::Msg),
}

enum Route {
    Main,
    Event,
    Login,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(url_changed) => {
            let mut new_url = url_changed.0;
            let previous_url = Some(model.current_url.clone());
            log!(format!(
                "go to url={}, previous={}",
                new_url.clone(),
                model.current_url.clone()
            ));
            model.current_url = new_url.clone();
            model.page = page_from_url(&mut new_url, previous_url, &model.context, orders);
        }
        Msg::Main(main_msg) => {
            if let Page::Main(main_model) = &mut model.page {
                pages::main::update(
                    main_msg,
                    main_model,
                    &mut model.context,
                    &mut orders.proxy(Msg::Main),
                );
            }
        }
        Msg::Event(event_msg) => {
            if let Page::Event(event_model) = &mut model.page {
                pages::event::update(
                    event_msg,
                    event_model,
                    &mut model.context,
                    &mut orders.proxy(Msg::Event),
                );
            }
        }
        Msg::Login(login_msg) => {
            if let Page::Login(login_model) = &mut model.page {
                match login_msg {
                    pages::login::Msg::Public(msg) => match msg {
                        pages::login::PublicMsg::LoggedIn(username, url) => {
                            model.context.username = Some(username);
                            match url {
                                Some(url) => change_url(url, orders),
                                None => { /* noting to do */ }
                            }
                        }
                    },
                    pages::login::Msg::Private(msg) => {
                        pages::login::update(
                            msg,
                            login_model,
                            &mut model.context,
                            &mut orders.proxy(Msg::Login),
                        );
                    }
                }
            }
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::Main(model) => pages::main::view(model).map_msg(Msg::Main),
        Page::Event(model) => pages::event::view(model).map_msg(Msg::Event),
        Page::Login(model) => pages::login::view(model).map_msg(Msg::Login),
    }
}

fn change_url(url: Url, orders: &mut impl Orders<Msg>) {
    // TODO: update address bar
    orders.notify(UrlChanged(url));
}
