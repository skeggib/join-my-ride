use seed::{prelude::*, *};

pub fn init(placeholder: String) -> Model {
    Model {
        placeholder: placeholder,
        value: "".into(),
    }
}

pub struct Model {
    placeholder: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub enum Msg {
    ValueChange(String),
}

pub fn update(model: &Model, msg: &Msg) -> Model {
    match msg {
        Msg::ValueChange(new_value) => Model {
            placeholder: model.placeholder.clone(),
            value: new_value.clone(),
        },
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    input![
        attrs![At::Value => model.value; At::Placeholder => model.placeholder],
        input_ev(Ev::Input, |value| { Msg::ValueChange(value) })
    ]
}
