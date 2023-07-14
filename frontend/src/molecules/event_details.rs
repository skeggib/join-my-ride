use common::Event;
use seed::{prelude::*, *};

pub fn init(event: Event) -> Model {
    Model { event: event }
}

pub struct Model {
    event: Event,
}

pub fn view(model: &Model) -> Node<()> {
    let participants_divs: Vec<Node<()>> = model
        .event
        .participants
        .iter()
        .map(|participant| div![participant.clone()])
        .collect();
    div![
        h2!("event"),
        div![&model.event.name],
        div![participants_divs],
    ]
}
