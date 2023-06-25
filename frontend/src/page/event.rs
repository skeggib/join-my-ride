use crate::component::event;
use crate::rest::get_json;
use common::{Event, Id};
use seed::{prelude::*, *};
use std::{future::Future, str::FromStr};

fn id_from_url(url: &mut Url) -> Result<Id, String> {
    Id::from_str(
        url.next_path_part()
            .ok_or("expected an ID in the URL".to_owned())?,
    )
    .map_err(|err| err.to_string())
}

pub fn init(url: &mut Url, orders: &mut impl Orders<Msg>) -> Model {
    match id_from_url(url) {
        Ok(id) => {
            event::request_event(id, &mut orders.proxy(Msg::Event));
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
    pub event: event::Model,
}

#[derive(Clone)]
pub enum Msg {
    Event(event::Msg),
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
        Msg::Event(event_msg) => match event_msg {
            event::Msg::OnGetEventResponse(event) => Model::Loaded(Loaded { event: event }),
            event::Msg::Error(error) => Model::Failed(error),
            event::Msg::JoinEventClick => todo!(),
        },
    }
}

fn update_loaded(
    msg: Msg,
    loaded: &Loaded,
    orders: &mut impl Orders<Msg>,
) -> Result<Loaded, String> {
    Ok(Loaded {
        event: match msg.clone() {
            Msg::Event(msg_event) => event::update(
                msg_event,
                loaded.event.clone(),
                &mut orders.proxy(Msg::Event),
            )?,
            _ => loaded.event.clone(),
        },
    })
}

pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::Loading => div!["loading..."],
        Model::Loaded(loaded) => div![&event::view(&loaded.event).map_msg(Msg::Event)],
        Model::Failed(err) => div![err],
    }
}
