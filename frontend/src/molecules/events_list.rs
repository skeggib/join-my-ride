use common::Event;
use seed::{prelude::*, *};

pub fn init(events: Vec<Event>) -> Model {
    Model { events: events }
}

pub struct Model {
    events: Vec<Event>,
}

pub fn view(model: &Model) -> Node<()> {
    let event_divs: Vec<Node<()>> = model
        .events
        .iter()
        .map(|event| {
            div![a![
                attrs! {At::Href => format!("/event/{}", event.id.to_string())},
                event.name.clone()
            ]]
        })
        .collect();
    div![h2!("all events"), event_divs,]
}
