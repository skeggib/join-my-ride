use seed::{prelude::*, *};

pub fn init(name: String) -> Model {
    Model { name: name }
}

pub struct Model {
    name: String,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Click,
}

pub fn view(model: &Model, is_active: bool) -> Node<Msg> {
    button![
        model.name.clone(),
        attrs![At::Disabled => (!is_active).as_at_value()],
        ev(Ev::Click, |_| Msg::Click)
    ]
}
