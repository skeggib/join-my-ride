use crate::{event_list, event_publication};
use seed::{prelude::*, *};

pub fn init(_: &mut Url, orders: &mut impl Orders<Msg>) -> Model {
    event_list::request_events(&mut orders.proxy(Msg::EventList));
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
    pub event_list: event_list::Model,
    pub event_publication_form: event_publication::Model,
}

#[derive(Clone)]
pub enum Msg {
    EventList(event_list::Msg),
    EventPublication(event_publication::Msg),
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
        Msg::EventList(event_list_msg) => match event_list_msg {
            event_list::Msg::OnGetEventsResponse(events) => Model::Loaded(Loaded {
                event_list: events,
                event_publication_form: event_publication::Model::Empty,
            }),
            event_list::Msg::Error(error) => Model::Failed(error),
        },
        Msg::EventPublication(_) => {
            error!("received a publication form message while loading");
            Model::Loading
        }
    }
}

fn update_loaded(
    msg: Msg,
    loaded: &Loaded,
    orders: &mut impl Orders<Msg>,
) -> Result<Loaded, String> {
    match msg {
        Msg::EventPublication(event_publication::Msg::OnEventPublished) => {
            event_list::request_events(&mut orders.proxy(Msg::EventList));
        }
        _ => {}
    };
    Ok(Loaded {
        event_list: match msg.clone() {
            Msg::EventList(msg_event_list) => event_list::update(msg_event_list)?,
            _ => loaded.event_list.clone(),
        },
        event_publication_form: match msg {
            Msg::EventPublication(msg_event_publication_form) => event_publication::update(
                msg_event_publication_form,
                &loaded.event_publication_form,
                &mut orders.proxy(Msg::EventPublication),
            )?,
            _ => loaded.event_publication_form.clone(),
        },
    })
}

pub fn view(model: &Model) -> Node<Msg> {
    match &model {
        Model::Loading => div!["loading..."],
        Model::Loaded(loaded_state) => {
            div![
                event_list::view(&loaded_state.event_list).map_msg(Msg::EventList),
                event_publication::view(&loaded_state.event_publication_form)
                    .map_msg(Msg::EventPublication),
            ]
        }
        Model::Failed(error) => div![error],
    }
}
