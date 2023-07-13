use crate::app::Context;
use crate::molecules::event_publication_form;
use crate::molecules::events_list;
use crate::orders::perform_cmd;
use common::Event;
use seed::{prelude::*, *};

pub fn init(_: &mut Url, orders: &mut impl Orders<Msg>) -> Model {
    request_events(orders);
    Model {
        state: State::Loading,
    }
}

type ErrorMessage = String;

#[derive(Clone)]
pub struct Model {
    state: State,
}

#[derive(Clone)]
pub enum State {
    Loading,
    Loaded(Loaded),
    Failed(ErrorMessage),
}

#[derive(Clone)]
pub struct Loaded {
    pub event_list: events_list::Model,
    pub event_publication_form: event_publication_form::Model,
}

impl Loaded {
    fn new(events: Vec<Event>, context: &Context) -> Loaded {
        Loaded {
            event_list: events_list::init(events),
            event_publication_form: event_publication_form::init(),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    OnGetEventsResponse(Vec<Event>),
    EventPublication(event_publication_form::Msg),
    Error(String),
}

pub fn update(msg: Msg, model: &mut Model, context: &mut Context, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::OnGetEventsResponse(events) => {
            on_get_events_response_msg(events, model, context, orders)
        }
        Msg::EventPublication(msg) => event_publication_form_msg(msg, model, context, orders),
        Msg::Error(err) => model.state = State::Failed(err),
    }
}

fn on_get_events_response_msg(
    events: Vec<Event>,
    model: &mut Model,
    context: &mut Context,
    _: &mut impl Orders<Msg>,
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
    context: &mut Context,
    orders: &mut impl Orders<Msg>,
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

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!(a![attrs![At::Href => "/"], "join my ride"]),
        match &model.state {
            State::Loading => div!["loading..."],
            State::Loaded(loaded_state) => {
                div![
                    h2!("all events"),
                    events_list::view(&loaded_state.event_list).map_msg(|_| {
                        // TODO: remove this map_msg since events_list does not have any
                        Msg::Error("unexpected msg from events list".to_owned())
                    }),
                    h2!("publish an event"),
                    event_publication_form::view(&loaded_state.event_publication_form)
                        .map_msg(Msg::EventPublication),
                ]
            }
            State::Failed(error) => div![error],
        }
    ]
}

pub fn request_events(orders: &mut impl Orders<Msg>) {
    log!("get all events");
    perform_cmd(orders, async {
        match common::api::get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
            Err(error) => Msg::Error(error),
        }
    });
}
