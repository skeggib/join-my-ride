use crate::app::Context;
use crate::atoms::button;
use crate::molecules::event_details;
use crate::molecules::login_bar;
use crate::orders::perform_cmd;
use crate::orders::IMyOrders;
use common::{Event, Id};
use seed::{prelude::*, *};
use std::str::FromStr;

fn id_from_url(url: &mut Url) -> Result<Id, String> {
    Id::from_str(
        url.next_path_part()
            .ok_or("expected an ID in the URL".to_owned())?,
    )
    .map_err(|err| err.to_string())
}

pub fn request_event(id: Id, orders: &mut impl IMyOrders<Msg>) {
    log!("get event {}", id);
    perform_cmd(orders, async move {
        match common::api::get_event(id).await {
            Ok(event) => Msg::OnGetEventResponse(event),
            Err(error) => Msg::Error(error),
        }
    });
}

pub fn join_event(id: Id, orders: &mut impl IMyOrders<Msg>) {
    log!("join event {}", id);
    orders.perform_cmd(async move {
        match common::api::join_event(id).await {
            Ok(_) => match common::api::get_event(id).await {
                Ok(event) => Msg::OnGetEventResponse(event),
                Err(error) => Msg::Error(error),
            },
            Err(error) => Msg::Error(error),
        }
    });
}

pub fn init(url: &mut Url, orders: &mut impl IMyOrders<Msg>) -> Model {
    match id_from_url(url) {
        Ok(id) => {
            request_event(id, orders);
            Model {
                state: State::Loading,
            }
        }
        Err(err) => Model {
            state: State::Failed(err),
        },
    }
}

type ErrorMessage = String;

pub struct Model {
    state: State,
}

pub enum State {
    Loading,
    Loaded(Loaded),
    Failed(ErrorMessage),
}

pub struct Loaded {
    event: Event,
    event_details: event_details::Model,
    join_button: button::Model,
    login_bar: login_bar::Model,
}

impl Loaded {
    fn new(event: Event, context: &Context) -> Loaded {
        Loaded {
            event: event.clone(),
            event_details: event_details::init(event),
            join_button: button::init("join".into()),
            login_bar: login_bar::init(context.username.clone()),
        }
    }
}

pub enum Msg {
    OnGetEventResponse(Event),
    Error(String),
    JoinButton(button::Msg),
    LoginBar(login_bar::Msg),
}

pub fn update(
    msg: Msg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl IMyOrders<Msg>,
) {
    match msg {
        Msg::OnGetEventResponse(event) => on_get_event_response_msg(event, model, context, orders),
        Msg::Error(err) => model.state = State::Failed(err),
        Msg::JoinButton(msg) => join_button_msg(msg, model, orders),
        Msg::LoginBar(msg) => login_bar_msg(msg, model, context, orders),
    }
}

fn on_get_event_response_msg(
    event: Event,
    model: &mut Model,
    context: &mut Context,
    _: &mut impl IMyOrders<Msg>,
) {
    match &mut model.state {
        State::Loading => model.state = State::Loaded(Loaded::new(event, context)),
        State::Loaded(_loaded) => model.state = State::Loaded(Loaded::new(event, context)),
        State::Failed(_) => { /* nothing to do */ }
    }
}

fn join_button_msg(msg: button::Msg, model: &mut Model, orders: &mut impl IMyOrders<Msg>) {
    match &mut model.state {
        State::Loading => error!("received a join button msg while loading"),
        State::Loaded(loaded) => match msg {
            button::Msg::Click => join_event(loaded.event.id, orders),
        },
        State::Failed(_) => error!("received a join button msg while failed"),
    }
}

fn login_bar_msg(
    msg: login_bar::Msg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl IMyOrders<Msg>,
) {
    match &mut model.state {
        State::Loading => error!("received a login bar msg while loading"),
        State::Loaded(loaded) => match msg {
            login_bar::Msg::Public(msg) => match msg {
                login_bar::PublicMsg::SignedOut => { /* nothing to do */ }
            },
            login_bar::Msg::Private(msg) => login_bar::update(
                msg,
                &mut loaded.login_bar,
                context,
                &mut orders.proxy(Msg::LoginBar),
            ),
        },
        State::Failed(_) => error!("received a login bar msg while failed"),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!(a![attrs![At::Href => "/"], "join my ride"]),
        match &model.state {
            State::Loading => div!["loading..."],
            State::Loaded(loaded) => div![
                login_bar::view(&loaded.login_bar).map_msg(Msg::LoginBar),
                &event_details::view(&loaded.event_details).map_msg(|_| {
                    // TODO: remove this map_msg since events_list does not have any
                    Msg::Error("unexpected msg from event details".to_owned())
                }),
                button::view(
                    &loaded.join_button,
                    !loaded.event.participants.contains("valid_user")
                )
                .map_msg(Msg::JoinButton)
            ],
            State::Failed(err) => div![err],
        }
    ]
}
