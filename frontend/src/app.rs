use std::rc::Rc;

use crate::{
    orders::{IMyOrders, MyOrders, OrdersImplementation},
    pages,
};
use common::{api::BackendApi, rest::RestBackend};
use seed::{
    app::OrdersContainer,
    prelude::{subs::UrlChanged, *},
};

pub fn init(url: Url, orders: &mut OrdersContainer<SeedMsg, Model, Node<SeedMsg>>) -> Model {
    let mut my_orders = MyOrders::new(OrdersImplementation::<SeedMsg, SeedMsg>::Proxy(
        orders.proxy(|msg| match msg {
            SeedMsg::SeedUrlChanged(msg) => SeedMsg::SeedUrlChanged(msg),
            SeedMsg::Main(msg) => SeedMsg::Main(msg),
            SeedMsg::Event(msg) => SeedMsg::Event(msg),
            SeedMsg::Login(msg) => SeedMsg::Login(msg),
        }),
    ));
    testable_init(url, &mut my_orders, Rc::new(RestBackend {}))
}

pub fn testable_init(
    mut url: Url,
    orders: &mut impl IMyOrders<SeedMsg>,
    backend: Rc<dyn BackendApi>,
) -> Model {
    orders.subscribe(SeedMsg::SeedUrlChanged);
    let context = Context {
        username: None,
        backend: backend,
    };
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
    orders: &mut impl IMyOrders<SeedMsg>,
) -> Page {
    match parse_url(url) {
        Route::Main => Page::Main(pages::main::init(
            url,
            context,
            &mut orders.proxy(SeedMsg::Main),
        )),
        Route::Event => Page::Event(pages::event::init(
            url,
            context,
            &mut orders.proxy(SeedMsg::Event),
        )),
        Route::Login => Page::Login(pages::login::init(
            url,
            previous_url,
            context,
            &mut orders.proxy(SeedMsg::Login),
        )),
    }
}

pub enum Page {
    Main(pages::main::Model),
    Event(pages::event::Model),
    Login(pages::login::Model),
}

pub struct Context {
    pub username: Option<String>,
    pub backend: Rc<dyn BackendApi>,
}

pub struct Model {
    pub page: Page,
    pub current_url: Url,
    pub context: Context,
}

#[derive(Clone, Debug)]
pub enum SeedMsg {
    SeedUrlChanged(subs::UrlChanged),
    Main(pages::main::Msg),
    Event(pages::event::Msg),
    Login(pages::login::Msg),
}

enum Route {
    Main,
    Event,
    Login,
}

pub fn update(
    msg: SeedMsg,
    model: &mut Model,
    orders: &mut OrdersContainer<SeedMsg, Model, Node<SeedMsg>>,
) {
    let mut my_orders = MyOrders::new(OrdersImplementation::<SeedMsg, SeedMsg>::Proxy(
        orders.proxy(|msg| match msg {
            SeedMsg::SeedUrlChanged(msg) => SeedMsg::SeedUrlChanged(msg),
            SeedMsg::Main(msg) => SeedMsg::Main(msg),
            SeedMsg::Event(msg) => SeedMsg::Event(msg),
            SeedMsg::Login(msg) => SeedMsg::Login(msg),
        }),
    ));
    testable_update(msg, model, &mut my_orders);
}

pub fn testable_update(msg: SeedMsg, model: &mut Model, orders: &mut impl IMyOrders<SeedMsg>) {
    match msg {
        SeedMsg::SeedUrlChanged(url_changed) => {
            let mut new_url = url_changed.0;
            let previous_url = Some(model.current_url.clone());
            // TODO: refactor this to use a logging service
            // log!(format!(
            //     "go to url={}, previous={}",
            //     new_url.clone(),
            //     model.current_url.clone()
            // ));
            model.current_url = new_url.clone();
            model.page = page_from_url(&mut new_url, previous_url, &model.context, orders);
        }
        SeedMsg::Main(main_msg) => {
            if let Page::Main(main_model) = &mut model.page {
                pages::main::update(
                    main_msg,
                    main_model,
                    &mut model.context,
                    &mut orders.proxy(SeedMsg::Main),
                );
            }
        }
        SeedMsg::Event(event_msg) => {
            if let Page::Event(event_model) = &mut model.page {
                pages::event::update(
                    event_msg,
                    event_model,
                    &mut model.context,
                    &mut orders.proxy(SeedMsg::Event),
                );
            }
        }
        SeedMsg::Login(login_msg) => {
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
                            &mut orders.proxy(SeedMsg::Login),
                        );
                    }
                }
            }
        }
    }
}

pub fn view(model: &Model) -> Node<SeedMsg> {
    match &model.page {
        Page::Main(model) => pages::main::view(model).map_msg(SeedMsg::Main),
        Page::Event(model) => pages::event::view(model).map_msg(SeedMsg::Event),
        Page::Login(model) => pages::login::view(model).map_msg(SeedMsg::Login),
    }
}

fn change_url(url: Url, orders: &mut impl IMyOrders<SeedMsg>) {
    // TODO: update address bar
    orders.notify(UrlChanged(url));
}
