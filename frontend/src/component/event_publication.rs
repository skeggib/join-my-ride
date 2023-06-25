use crate::orders::perform_cmd;
use crate::rest::put_json;
use common::Event;
use seed::{prelude::*, *};

#[derive(Clone)]
pub enum Model {
    Empty,
    Typing(Form),
    Publishing(Form),
    Invalid((Option<Form>, String)),
}

#[derive(Clone)]
pub struct Form {
    event_name: String,
}

#[derive(Clone)]
pub enum Msg {
    PublishEventNameChange(String),
    PublishEventClick,
    OnEventPublished,
}

async fn put_event(name: String) -> Result<(), String> {
    let event = Event::new(name);
    put_json("/api/event", &event).await
}

fn publish_event(contents: &Form, orders: &mut impl Orders<Msg>) {
    log!("publish event");
    let name = contents.event_name.clone();
    perform_cmd(orders, async move {
        match put_event(name).await {
            Ok(_) => Msg::OnEventPublished,
            Err(error) => {
                error!(error);
                todo!()
            }
        }
    });
}

pub fn update(msg: Msg, form: &Model, orders: &mut impl Orders<Msg>) -> Result<Model, String> {
    match form {
        Model::Empty => update_empty(msg),
        Model::Typing(contents) => update_typing(msg, contents, orders),
        Model::Publishing(contents) => update_publishing(msg, contents, orders),
        Model::Invalid((contents, _)) => update_invalid(msg, contents),
    }
}

fn update_empty(msg: Msg) -> Result<Model, String> {
    match msg {
        Msg::PublishEventNameChange(name) => Ok(Model::Typing(Form { event_name: name })),
        Msg::PublishEventClick => Ok(Model::Invalid((None, "The name is required".into()))),
        Msg::OnEventPublished => {
            error!("should not happen");
            Ok(Model::Empty)
        }
    }
}

fn update_typing(
    msg: Msg,
    contents: &Form,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    match msg {
        Msg::PublishEventNameChange(name) => {
            let mut new_contents = contents.clone();
            new_contents.event_name = name;
            Ok(Model::Typing(new_contents))
        }
        Msg::PublishEventClick => {
            if contents.event_name.is_empty() {
                Ok(Model::Invalid((
                    Some(contents.clone()),
                    "The name is required".into(),
                )))
            } else {
                publish_event(contents, orders);
                Ok(Model::Publishing(contents.clone()))
            }
        }
        Msg::OnEventPublished => {
            error!("should not happen");
            Ok(Model::Empty)
        }
    }
}

fn update_publishing(
    msg: Msg,
    contents: &Form,
    orders: &mut impl Orders<Msg>,
) -> Result<Model, String> {
    match msg {
        Msg::PublishEventNameChange(name) => {
            let mut new_contents = contents.clone();
            new_contents.event_name = name;
            Ok(Model::Publishing(new_contents))
        }
        Msg::PublishEventClick => {
            publish_event(contents, orders);
            Ok(Model::Publishing(contents.clone()))
        }
        Msg::OnEventPublished => Ok(Model::Empty),
    }
}

fn update_invalid(msg: Msg, maybe_contents: &Option<Form>) -> Result<Model, String> {
    match maybe_contents {
        Some(contents) => match msg {
            Msg::PublishEventNameChange(name) => {
                let mut new_contents = contents.clone();
                new_contents.event_name = name;
                Ok(Model::Typing(new_contents))
            }
            Msg::PublishEventClick => Ok(Model::Typing(contents.clone())),
            Msg::OnEventPublished => {
                error!("should not happen");
                Ok(Model::Empty)
            }
        },
        None => Ok(Model::Empty),
    }
}

pub fn view(form: &Model) -> Node<Msg> {
    let value = match &form {
        Model::Empty => "".into(),
        Model::Typing(contents) => contents.event_name.clone(),
        Model::Publishing(contents) => contents.event_name.clone(),
        Model::Invalid((maybe_contents, _)) => match maybe_contents {
            Some(contents) => contents.event_name.clone(),
            None => "".into(),
        },
    };
    let input = input![
        attrs![At::Value => value],
        input_ev(Ev::Input, |value| { Msg::PublishEventNameChange(value) })
    ];
    let is_form_ready_for_publishing = match form {
        Model::Empty => false,
        Model::Typing(_) => true,
        Model::Publishing(_) => false,
        Model::Invalid(_) => false,
    };
    div![
        match &form {
            Model::Invalid((_, error)) => div![input, error],
            _ => div![input],
        },
        button![
            "publish event",
            attrs![At::Disabled => (!is_form_ready_for_publishing).as_at_value()],
            ev(Ev::Click, |_| Msg::PublishEventClick)
        ]
    ]
}
