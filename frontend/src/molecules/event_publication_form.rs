use crate::atoms::{button, input};
use crate::orders::perform_cmd;
use seed::{prelude::*, *};

pub fn init() -> Model {
    Model {
        state: State::Typing,
        event_name: input::init("name".into()),
        publish_button: button::init("publish".to_owned()),
    }
}

pub struct Model {
    state: State,
    event_name: input::Model,
    publish_button: button::Model,
}

pub enum State {
    Typing,
    Publishing,
    Invalid(String),
}

pub enum Msg {
    Public(PublicMsg),
    Private(PrivateMsg),
}

pub enum PublicMsg {
    EventPublished,
}

pub enum PrivateMsg {
    EventName(input::Msg),
    PublishButton(button::Msg),
}

fn publish_event(model: &Model, orders: &mut impl Orders<Msg>) {
    log!("publish event");
    let name = model.event_name.value.clone();
    perform_cmd(orders, async move {
        match common::api::publish_event(name).await {
            Ok(_) => Msg::Public(PublicMsg::EventPublished),
            Err(error) => {
                error!(error);
                todo!()
            }
        }
    });
}

pub fn update(msg: PrivateMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match model.state {
        State::Typing => update_typing(msg, model, orders),
        State::Publishing => update_publishing(msg, model, orders),
        State::Invalid(_) => update_invalid(msg, model, orders),
    }
}

fn update_typing(msg: PrivateMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        PrivateMsg::EventName(msg) => {
            model.event_name = input::update(&model.event_name, &msg);
        }
        PrivateMsg::PublishButton(button::Msg::Click) => {
            if model.event_name.value.is_empty() {
                model.state = State::Invalid("The name is required".into());
            } else {
                publish_event(model, orders);
                model.state = State::Publishing;
            }
        }
    }
}

fn update_publishing(msg: PrivateMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        PrivateMsg::EventName(msg) => {
            model.event_name = input::update(&model.event_name, &msg);
        }
        PrivateMsg::PublishButton(button::Msg::Click) => {
            error!("received a publish button click msg while publishing")
        }
    }
}

fn update_invalid(msg: PrivateMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        PrivateMsg::EventName(msg) => {
            model.state = State::Typing;
            model.event_name = input::update(&model.event_name, &msg);
        }
        PrivateMsg::PublishButton(button::Msg::Click) => {
            error!("received a publish button click msg while being an invalid form")
        }
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
