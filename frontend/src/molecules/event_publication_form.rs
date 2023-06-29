use crate::atoms::{button, input};
use crate::orders::perform_cmd;
use crate::rest::put_json;
use common::Event;
use seed::{prelude::*, *};

pub fn init() -> Model {
    Model {
        state: State::Typing,
        event_name: input::init("name".into()),
        publish_button: button::init("publish".to_owned()),
    }
}

#[derive(Clone)]
pub struct Model {
    state: State,
    event_name: input::Model,
    publish_button: button::Model,
}

#[derive(Clone)]
pub enum State {
    Typing,
    Publishing,
    Invalid(String),
}

#[derive(Clone)]
pub enum Msg {
    Public(PublicMsg),
    Private(PrivateMsg),
}

#[derive(Clone)]
pub enum PublicMsg {
    EventPublished,
}

#[derive(Clone)]
pub enum PrivateMsg {
    EventName(input::Msg),
    PublishButton(button::Msg),
}

async fn put_event(name: String) -> Result<(), String> {
    let event = Event::new(name);
    put_json("/api/event", &event).await
}

fn publish_event(model: &Model, orders: &mut impl Orders<Msg>) {
    log!("publish event");
    let name = model.event_name.value.clone();
    perform_cmd(orders, async move {
        match put_event(name).await {
            Ok(_) => Msg::Public(PublicMsg::EventPublished),
            Err(error) => {
                error!(error);
                todo!()
            }
        }
    });
}

pub fn update(
    msg: PrivateMsg,
    model: &Model,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    match model.state {
        State::Typing => update_typing(msg, model, orders),
        State::Publishing => update_publishing(msg, model),
        State::Invalid(_) => update_invalid(msg, model),
    }
}

fn update_typing(
    msg: PrivateMsg,
    model: &Model,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    match msg {
        PrivateMsg::EventName(msg) => {
            let mut new_model = model.clone();
            new_model.event_name = input::update(&model.event_name, &msg);
            Ok(new_model)
        }
        PrivateMsg::PublishButton(button::Msg::Click) => {
            if model.event_name.value.is_empty() {
                let mut new_model = model.clone();
                new_model.state = State::Invalid("The name is required".into());
                Ok(new_model)
            } else {
                publish_event(model, orders);
                let mut new_model = model.clone();
                new_model.state = State::Publishing;
                Ok(new_model)
            }
        }
    }
}

fn update_publishing(msg: PrivateMsg, model: &Model) -> Result<Model, String> {
    match msg {
        PrivateMsg::EventName(msg) => {
            let mut new_model = model.clone();
            new_model.event_name = input::update(&model.event_name, &msg);
            Ok(new_model)
        }
        PrivateMsg::PublishButton(button::Msg::Click) => Ok(model.clone()),
    }
}

fn update_invalid(msg: PrivateMsg, model: &Model) -> Result<Model, String> {
    match msg {
        PrivateMsg::EventName(msg) => {
            let mut new_model = model.clone();
            new_model.state = State::Typing;
            new_model.event_name = input::update(&model.event_name, &msg);
            Ok(new_model)
        }
        PrivateMsg::PublishButton(button::Msg::Click) => Ok(model.clone()),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let input = input::view(&model.event_name)
        .map_msg(PrivateMsg::EventName)
        .map_msg(Msg::Private);
    let is_form_ready_for_publishing = match model.state {
        State::Typing => !model.event_name.value.is_empty(),
        State::Publishing => false,
        State::Invalid(_) => false,
    };
    div![
        match &model.state {
            State::Invalid(error) => div![input, error],
            _ => div![input],
        },
        button::view(&model.publish_button, is_form_ready_for_publishing)
            .map_msg(PrivateMsg::PublishButton)
            .map_msg(Msg::Private)
    ]
}
