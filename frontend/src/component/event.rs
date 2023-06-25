use crate::rest::get_json;
use common::Event;
use seed::{prelude::*, *};
use std::future::Future;

pub type Model = Event;

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

#[derive(Clone)]
pub enum Msg {
    OnGetEventResponse(Event),
    Error(String),
}

pub fn update(msg: Msg) -> Result<Model, String> {
    match msg {
        Msg::OnGetEventResponse(event) => Ok(event),
        Msg::Error(error) => Err(error),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![&model.name]
}
