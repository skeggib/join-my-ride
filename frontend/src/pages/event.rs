use crate::app::Context;
use crate::orders::perform_cmd;
use crate::{atoms::button, molecules::event_details};
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

pub fn request_event(id: Id, orders: &mut impl Orders<Msg>) {
    log!("get event {}", id);
    perform_cmd(orders, async move {
        match common::api::get_event(id).await {
            Ok(event) => Msg::OnGetEventResponse(event),
            Err(error) => Msg::Error(error),
        }
    });
}

pub fn join_event(id: Id, orders: &mut impl Orders<Msg>) {
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

pub fn init(url: &mut Url, orders: &mut impl Orders<Msg>) -> Model {
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
    event: Event,
    event_details: event_details::Model,
    join_button: button::Model,
}

impl Loaded {
    fn new(event: Event) -> Loaded {
        Loaded {
            event: event.clone(),
            event_details: event_details::init(event),
            join_button: button::init("join".into()),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    OnGetEventResponse(Event),
    Error(String),
    JoinButton(button::Msg),
}

pub fn update(msg: Msg, model: &mut Model, context: &mut Context, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::OnGetEventResponse(event) => on_get_event_response_msg(event, model, orders),
        Msg::Error(err) => model.state = State::Failed(err),
        Msg::JoinButton(msg) => join_button_msg(msg, model, orders),
    }
}

fn on_get_event_response_msg(event: Event, model: &mut Model, _: &mut impl Orders<Msg>) {
    match &mut model.state {
        State::Loading => model.state = State::Loaded(Loaded::new(event)),
        State::Loaded(loaded) => model.state = State::Loaded(Loaded::new(event)),
        State::Failed(_) => { /* nothing to do */ }
    }
}

fn join_button_msg(msg: button::Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match &mut model.state {
        State::Loading => error!("received a join button msg while loading"),
        State::Loaded(loaded) => match msg {
            button::Msg::Click => join_event(loaded.event.id, orders),
        },
        State::Failed(_) => error!("received a join button msg while failed"),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!(a![attrs![At::Href => "/"], "join my ride"]),
        match &model.state {
            State::Loading => div!["loading..."],
            State::Loaded(loaded) => div![
                h2!("event"),
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
