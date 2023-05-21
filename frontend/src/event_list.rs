use crate::rest::get_json;
use common::Event;
use seed::{prelude::*, *};
use std::future::Future;

pub type Model = Vec<Event>;

async fn get_events() -> Result<Vec<Event>, String> {
    get_json::<Vec<Event>>("/api/events").await
}

pub fn request_events(orders: &mut impl Orders<Msg>) {
    log!("get all events");
    perform_cmd(orders, async {
        match get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
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
    OnGetEventsResponse(Vec<Event>),
    Error(String),
}

pub fn update(msg: Msg) -> Result<Model, String> {
    match msg {
        Msg::OnGetEventsResponse(events) => Ok(events),
        Msg::Error(error) => Err(error),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let event_divs: Vec<Node<Msg>> = model.iter().map(|event| div![event.name.clone()]).collect();
    div![event_divs]
}
