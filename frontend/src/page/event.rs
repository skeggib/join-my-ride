use crate::rest::get_json;
use common::{Event, Id};
use seed::{prelude::*, *};
use std::{future::Future, str::FromStr};

async fn get_event(id: Id) -> Result<Event, String> {
    get_json::<Event>(&format!("/api/event/{}", id)).await
}

pub fn request_event(id: Id, orders: &mut impl Orders<Msg>) {
    log!("get event {}", id);
    perform_cmd(orders, async move {
        match get_event(id).await {
            Ok(event) => Msg::OnGetEventResponse(event),
            Err(error) => Msg::Error(error),
        }
    });
}

// TODO: move to common code
fn perform_cmd(orders: &mut impl Orders<Msg>, cmd: impl Future<Output = Msg> + 'static) {
    orders.perform_cmd(cmd);
}

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
    pub event: Event,
}

#[derive(Clone)]
pub enum Msg {
    OnGetEventResponse(Event),
    Error(String),
}

pub fn update(msg: Msg, _: &Model, _: &mut impl Orders<Msg>) -> Model {
    match msg {
        Msg::OnGetEventResponse(event) => Model::Loaded(Loaded { event: event }),
        Msg::Error(err) => Model::Failed(err),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::Loading => div!["loading..."],
        Model::Loaded(loaded) => div![&loaded.event.name],
        Model::Failed(err) => div![err],
    }
}
