use crate::molecules::event_publication_form;
use crate::molecules::events_list;
use crate::orders::perform_cmd;
use common::Event;
use seed::{prelude::*, *};

pub fn init(_: &mut Url, orders: &mut impl Orders<Msg>) -> Model {
    request_events(orders);
    Model::Loading
}

type ErrorMessage = String;

#[derive(Clone)]
pub enum Model {
    Loading,
    Loaded(Loaded),
    Failed(ErrorMessage),
}

#[derive(Clone)]
pub struct Loaded {
    pub event_list: events_list::Model,
    pub event_publication_form: event_publication_form::Model,
}

#[derive(Clone)]
pub enum Msg {
    OnGetEventsResponse(Vec<Event>),
    EventPublication(event_publication_form::Msg),
    Error(String),
}

pub fn update(msg: Msg, model: &Model, orders: &mut impl Orders<Msg>) -> Model {
    match model {
        Model::Loading => update_loading(msg),
        Model::Loaded(loaded) => match update_loaded(msg, loaded, orders) {
            Ok(new_loaded) => Model::Loaded(new_loaded),
            Err(error) => Model::Failed(error),
        },
        Model::Failed(failed) => Model::Failed(failed.into()),
    }
}

fn update_loading(msg: Msg) -> Model {
    match msg {
        Msg::OnGetEventsResponse(events) => Model::Loaded(Loaded {
            event_list: events_list::init(events),
            event_publication_form: event_publication_form::init(),
        }),
        Msg::EventPublication(_) => {
            error!("received a publication form message while loading");
            Model::Loading
        }
        Msg::Error(error) => Model::Failed(error),
    }
}

fn update_loaded(
    msg: Msg,
    loaded: &Loaded,
    orders: &mut impl Orders<Msg>,
) -> Result<Loaded, String> {
    match msg {
        Msg::OnGetEventsResponse(events) => Ok(Loaded {
            event_list: events_list::init(events),
            event_publication_form: loaded.event_publication_form.clone(),
        }),
        Msg::EventPublication(event_publication_msg) => match event_publication_msg {
            event_publication_form::Msg::Public(public_msg) => match public_msg {
                event_publication_form::PublicMsg::EventPublished => {
                    request_events(orders);
                    Ok(Loaded {
                        event_list: loaded.event_list.clone(),
                        event_publication_form: event_publication_form::init(),
                    })
                }
            },
            event_publication_form::Msg::Private(msg) => Ok(Loaded {
                event_list: loaded.event_list.clone(),
                event_publication_form: event_publication_form::update(
                    msg,
                    &loaded.event_publication_form,
                    &mut orders.proxy(Msg::EventPublication),
                )?,
            }),
        },
        Msg::Error(error) => Err(error),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![
        h1!("join my ride"),
        match &model {
            Model::Loading => div!["loading..."],
            Model::Loaded(loaded_state) => {
                div![
                    h2!("all events"),
                    events_list::view(&loaded_state.event_list).map_msg(|_| {
                        // TODO: remove this map_msg since events_list does not have any
                        Msg::Error("unexpected msg from events list".to_owned())
                    }),
                    h2!("publish an event"),
                    event_publication_form::view(&loaded_state.event_publication_form)
                        .map_msg(Msg::EventPublication),
                ]
            }
            Model::Failed(error) => div![error],
        }
    ]
}

pub fn request_events(orders: &mut impl Orders<Msg>) {
    log!("get all events");
    perform_cmd(orders, async {
        match common::api::get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
            Err(error) => Msg::Error(error),
        }
    });
}
