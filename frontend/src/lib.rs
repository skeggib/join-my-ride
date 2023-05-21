// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

mod event_list;
mod event_publication;
mod rest;

use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    event_list::request_events(&mut orders.proxy(Msg::EventList));
    Model {
        state: State::Loading,
    }
}

// ------ ------
//     Model
// ------ ------

type ErrorMessage = String;

#[derive(Clone)]
enum State {
    Loading,
    Loaded(LoadedState),
    Failed(ErrorMessage),
}

#[derive(Clone)]
struct LoadedState {
    event_list: event_list::Model,
    event_publication_form: event_publication::Model,
}

struct Model {
    state: State,
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
enum Msg {
    EventList(event_list::Msg),
    EventPublication(event_publication::Msg),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    model.state = update_state(msg, &model.state, orders);
}

fn update_state(msg: Msg, state: &State, orders: &mut impl Orders<Msg>) -> State {
    match state {
        State::Loading => update_state_loading(msg),
        State::Loaded(loaded) => match update_state_loaded(msg, loaded, orders) {
            Ok(new_loaded) => State::Loaded(new_loaded),
            Err(error) => State::Failed(error),
        },
        State::Failed(failed) => State::Failed(failed.into()),
    }
}

fn update_state_loading(msg: Msg) -> State {
    match msg {
        Msg::EventList(event_list_msg) => match event_list_msg {
            event_list::Msg::OnGetEventsResponse(events) => State::Loaded(LoadedState {
                event_list: events,
                event_publication_form: event_publication::Model::Empty,
            }),
            event_list::Msg::Error(error) => State::Failed(error),
        },
        Msg::EventPublication(_) => {
            error!("received a publication form message while loading");
            State::Loading
        }
    }
}

fn update_state_loaded(
    msg: Msg,
    loaded: &LoadedState,
    orders: &mut impl Orders<Msg>,
) -> Result<LoadedState, String> {
    match msg {
        Msg::EventPublication(event_publication::Msg::OnEventPublished) => {
            event_list::request_events(&mut orders.proxy(Msg::EventList));
        }
        _ => {}
    };
    Ok(LoadedState {
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

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    match &model.state {
        State::Loading => div!["loading..."],
        State::Loaded(loaded_state) => {
            div![
                event_list::view(&loaded_state.event_list).map_msg(Msg::EventList),
                event_publication::view(&loaded_state.event_publication_form)
                    .map_msg(Msg::EventPublication),
            ]
        }
        State::Failed(error) => div![error],
    }
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
