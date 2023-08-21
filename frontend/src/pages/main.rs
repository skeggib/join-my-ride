use crate::app::Context;
use crate::molecules::event_publication_form;
use crate::molecules::events_list;
use crate::molecules::login_bar;
use crate::orders::perform_cmd;
use crate::orders::IMyOrders;
use common::Event;
use seed::{prelude::*, *};

pub fn init(_: &mut Url, orders: &mut impl IMyOrders<Msg>) -> Model {
    request_events(orders);
    Model {
        state: State::Loading,
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
    pub event_list: events_list::Model,
    pub event_publication_form: event_publication_form::Model,
    pub login_bar: login_bar::Model,
}

impl Loaded {
    fn new(events: Vec<Event>, context: &Context) -> Loaded {
        Loaded {
            event_list: events_list::init(events),
            event_publication_form: event_publication_form::init(),
            login_bar: login_bar::init(context.username.clone()),
        }
    }
}

pub enum Msg {
    OnGetEventsResponse(Vec<Event>),
    EventPublication(event_publication_form::Msg),
    Error(String),
    LoginBar(login_bar::Msg),
}

pub fn update(
    msg: Msg,
    model: &mut Model,
    context: &mut Context,
    orders: &mut impl IMyOrders<Msg>,
) {
    match msg {
        Msg::OnGetEventsResponse(events) => {
            on_get_events_response_msg(events, model, context, orders)
        }
        Msg::EventPublication(msg) => event_publication_form_msg(msg, model, context, orders),
        Msg::Error(err) => model.state = State::Failed(err),
        Msg::LoginBar(msg) => login_bar_msg(msg, model, context, orders),
    }
}

fn on_get_events_response_msg(
    events: Vec<Event>,
    model: &mut Model,
    context: &mut Context,
    _: &mut impl IMyOrders<Msg>,
) {
    match &mut model.state {
        State::Loading => model.state = State::Loaded(Loaded::new(events, context)),
        State::Loaded(loaded) => loaded.event_list = events_list::init(events),
        State::Failed(_) => { /* nothing to do */ }
    }
}

fn event_publication_form_msg(
    msg: event_publication_form::Msg,
    model: &mut Model,
    _context: &mut Context,
    orders: &mut impl IMyOrders<Msg>,
) {
    match &mut model.state {
        State::Loading => error!("received an event publication form msg while loading"),
        State::Loaded(loaded) => match msg {
            event_publication_form::Msg::Public(msg) => match msg {
                event_publication_form::PublicMsg::EventPublished => {
                    request_events(orders);
                    loaded.event_publication_form = event_publication_form::init();
                }
            },
            event_publication_form::Msg::Private(msg) => event_publication_form::update(
                msg,
                &mut loaded.event_publication_form,
                &mut orders.proxy(Msg::EventPublication),
            ),
        },
        State::Failed(_) => error!("received an event publication form msg while failed"),
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
            State::Loaded(loaded_state) => {
                div![
                    login_bar::view(&loaded_state.login_bar).map_msg(Msg::LoginBar),
                    events_list::view(&loaded_state.event_list).map_msg(|_| {
                        // TODO: remove this map_msg since events_list does not have any
                        Msg::Error("unexpected msg from events list".to_owned())
                    }),
                    event_publication_form::view(&loaded_state.event_publication_form)
                        .map_msg(Msg::EventPublication),
                ]
            }
            State::Failed(error) => div![error],
        }
    ]
}

pub fn request_events(orders: &mut impl IMyOrders<Msg>) {
    log!("get all events");
    perform_cmd(orders, async {
        match common::api::get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
            Err(error) => Msg::Error(error),
        }
    });
}
