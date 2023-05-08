// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use gloo_net::http::Request;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let request = Request::new("/api/events");
    orders.perform_cmd(async {
        let response = async { request.send().await?.text().await }.await;
        match response {
            Ok(json_str) => match serde_json::from_str(json_str.as_str()) {
                Ok(events) => Msg::OnGetEventsResponse(events),
                Err(error) => Msg::Error(format!("{}", error)),
            },
            Err(error) => Msg::Error(format!("{}", error)),
        }
    });
    Model {
        events: vec![],
        error: None,
    }
}

// ------ ------
//     Model
// ------ ------

struct Model {
    events: Vec<Event>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Event {
    name: String,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    OnGetEventsResponse(Vec<Event>),
    Error(String),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::OnGetEventsResponse(events) => model.events = events,
        Msg::Error(error) => model.error = Some(error),
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    let event_divs: Vec<Node<Msg>> = model
        .events
        .iter()
        .map(|event| div![event.name.clone()])
        .collect();
    match &model.error {
        Some(error) => div![error],
        None => div![event_divs],
    }
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
