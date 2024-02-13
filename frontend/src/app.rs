use std::{rc::Rc, str::FromStr};

use crate::{
    orders::{IMyOrders, MyOrders, OrdersImplementation},
    pages,
};
use common::{api::BackendApi, rest::RestBackend};
use seed::{
    app::OrdersContainer,
    prelude::{subs::UrlChanged, *}, log,
};

pub fn init(url: Url, orders: &mut OrdersContainer<SeedMsg, Model, Node<SeedMsg>>) -> Model {
    orders.subscribe(SeedMsg::SeedUrlChanged);
    let proxy = orders.proxy::<AppMsg>(|msg| match msg {
        AppMsg::UrlChanged(url) => SeedMsg::SeedUrlChanged(UrlChanged(
            seed::browser::Url::from_str(&url.to_string()).unwrap(),
        )),
        msg => SeedMsg::AppMsg(msg),
    });
    let mut my_orders = MyOrders::new(OrdersImplementation::<AppMsg, SeedMsg>::Proxy(proxy));
    testable_init(crate::url::Url::from_str(&url.to_string()).unwrap(), &mut my_orders, Rc::new(RestBackend {}))
}

pub fn testable_init(
    mut url: crate::url::Url,
    orders: &mut impl IMyOrders<AppMsg>,
    backend: Rc<dyn BackendApi>,
) -> Model {
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

fn parse_url(url: &mut crate::url::Url) -> Route {
    // log!("parse url: {?}", url); // TODO: refactor this to use a logging service
    match url.next_path_part() {
        Some("event") => Route::Event,
        Some("login") => Route::Login,
        _ => Route::Main,
    }
}

fn page_from_url(
    url: &mut crate::url::Url,
    previous_url: Option<crate::url::Url>,
    context: &Context,
    orders: &mut impl IMyOrders<AppMsg>,
) -> Page {
    match parse_url(url) {
        Route::Main => Page::Main(pages::main::init(
            url,
            context,
            &mut orders.proxy(AppMsg::Main),
        )),
        Route::Event => Page::Event(pages::event::init(
            url,
            context,
            &mut orders.proxy(AppMsg::Event),
        )),
        Route::Login => Page::Login(pages::login::init(
            url,
            previous_url,
            context,
            &mut orders.proxy(AppMsg::Login),
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
    pub current_url: crate::url::Url,
    pub context: Context,
}

#[derive(Clone, Debug)]
pub enum SeedMsg {
    SeedUrlChanged(subs::UrlChanged),
    AppMsg(AppMsg),
}

#[derive(Clone, Debug)]
pub enum AppMsg {
    UrlChanged(crate::url::Url),
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
    let proxy = orders.proxy::<AppMsg>(|msg| match msg {
        AppMsg::UrlChanged(url) => SeedMsg::SeedUrlChanged(UrlChanged(
            seed::browser::Url::from_str(&url.to_string()).unwrap(),
        )),
        msg => SeedMsg::AppMsg(msg),
    });
    let mut my_orders = MyOrders::new(OrdersImplementation::<AppMsg, SeedMsg>::Proxy(proxy));
    testable_update(
        match msg {
            SeedMsg::SeedUrlChanged(url_changed) => {
                AppMsg::UrlChanged(crate::url::Url::from_str(&url_changed.0.to_string()).unwrap())
            }
            SeedMsg::AppMsg(msg) => msg,
        },
        model,
        &mut my_orders,
    );
}

pub fn testable_update(msg: AppMsg, model: &mut Model, orders: &mut impl IMyOrders<AppMsg>) {
    match msg {
        AppMsg::UrlChanged(mut new_url) => {
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
        AppMsg::Main(main_msg) => {
            if let Page::Main(main_model) = &mut model.page {
                pages::main::update(
                    main_msg,
                    main_model,
                    &mut model.context,
                    &mut orders.proxy(AppMsg::Main),
                );
            }
        }
        AppMsg::Event(event_msg) => {
            if let Page::Event(event_model) = &mut model.page {
                pages::event::update(
                    event_msg,
                    event_model,
                    &mut model.context,
                    &mut orders.proxy(AppMsg::Event),
                );
            }
        }
        AppMsg::Login(login_msg) => {
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
                            &mut orders.proxy(AppMsg::Login),
                        );
                    }
                }
            }
        }
    }
}

pub fn view(model: &Model) -> Node<SeedMsg> {
    testable_view(model).map_msg(SeedMsg::AppMsg)
}

pub fn testable_view(model: &Model) -> Node<AppMsg> {
    match &model.page {
        Page::Main(model) => {
            pages::main::view(model).map_msg(AppMsg::Main)
        }
        Page::Event(model) => {
            pages::event::view(model).map_msg(AppMsg::Event)
        }
        Page::Login(model) => {
            pages::login::view(model).map_msg(AppMsg::Login)
        }
    }
}

fn change_url(url: crate::url::Url, orders: &mut impl IMyOrders<AppMsg>) {
    // TODO: update address bar
    orders.notify(AppMsg::UrlChanged(crate::url::Url::from_str(&url.to_string()).unwrap()));
}
