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
            Model::Loading
        }
        Err(err) => Model::Failed(err),
    }
}

type ErrorMessage = String;

#[derive(Clone)]
pub enum Model {
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

#[derive(Clone)]
pub enum Msg {
    OnGetEventResponse(Event),
    Error(String),
    JoinButton(button::Msg),
}

pub fn update(msg: Msg, model: &Model, orders: &mut impl Orders<Msg>) -> Model {
    match model {
        Model::Loading => update_loading(msg),
        Model::Loaded(loaded) => match update_loaded(msg, loaded, orders) {
            Ok(new_loaded) => Model::Loaded(new_loaded),
            Err(error) => Model::Failed(error),
        },
        Model::Failed(failed) => Model::Failed(failed.into()),
    }
}

fn update_loading(msg: Msg) -> Model {
    match msg {
        Msg::OnGetEventResponse(event) => Model::Loaded(Loaded {
            event: event.clone(),
            event_details: event_details::init(event),
            join_button: button::init("join".to_owned()),
        }),
        Msg::Error(error) => Model::Failed(error),
        Msg::JoinButton(button::Msg::Click) => todo!(),
    }
}

fn update_loaded(
    msg: Msg,
    loaded: &Loaded,
    orders: &mut impl Orders<Msg>,
) -> Result<Loaded, String> {
    match msg {
        Msg::OnGetEventResponse(event) => Ok(Loaded {
            event: event.clone(),
            event_details: event_details::init(event),
            join_button: button::init("join".to_owned()),
        }),
        Msg::Error(error) => Err(error),
        Msg::JoinButton(_) => {
            join_event(loaded.event.id, orders);
            Ok(loaded.clone())
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!("join my ride"),
        match model {
            Model::Loading => div!["loading..."],
            Model::Loaded(loaded) => div![
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
            Model::Failed(err) => div![err],
        }
    ]
}
