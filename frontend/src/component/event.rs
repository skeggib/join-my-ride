use crate::rest::{get_json, put};
use common::{Event, Id};
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

pub fn join_event(id: Id, orders: &mut impl Orders<Msg>) {
    log!("join event {}", id);
    orders.perform_cmd(async move {
        match put(&format!("/api/join/{}", id)).await {
            Ok(_) => match get_event(id).await {
                Ok(event) => Msg::OnGetEventResponse(event),
                Err(error) => Msg::Error(error),
            },
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
    JoinEventClick,
}

pub fn update(msg: Msg, model: Model, orders: &mut impl Orders<Msg>) -> Result<Model, String> {
    match msg {
        Msg::OnGetEventResponse(event) => Ok(event),
        Msg::Error(error) => Err(error),
        Msg::JoinEventClick => {
            join_event(model.id, orders);
            Ok(model)
        }
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let participants_divs: Vec<Node<Msg>> = model
        .participants
        .iter()
        .map(|participant| div![participant.clone()])
        .collect();
    div![
        div![&model.name],
        div![participants_divs],
        div![button!["join", ev(Ev::Click, |_| Msg::JoinEventClick)]],
    ]
}
