use seed::{prelude::*, *};

pub fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {}
}

#[derive(Clone)]
pub struct Model {}

#[derive(Clone)]
pub enum Msg {}

pub fn update(msg: Msg, model: &Model, orders: &mut impl Orders<Msg>) -> Model {
    model.clone()
}

pub fn view(model: &Model) -> Node<Msg> {
    div![]
}
