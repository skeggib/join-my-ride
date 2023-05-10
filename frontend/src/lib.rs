// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use common::Event;
use gloo_net::http::Request;
use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------

async fn get(url: &str) -> Result<gloo_net::http::Response, String> {
    let request = Request::new(url);
    log!(format!("send request {:?}", request));
    request.send().await.map_err(|error| {
        let error = format!("could not send request {}", error.to_string());
        error!(error);
        error
    })
}

fn check_is_ok(response: gloo_net::http::Response) -> Result<gloo_net::http::Response, String> {
    if response.ok() {
        Ok(response)
    } else {
        let error = format!("server responded with code {}", response.status());
        error!(error);
        Err(error)
    }
}

async fn text(response: gloo_net::http::Response) -> Result<String, String> {
    response.text().await.map_err(|error| {
        let error = format!(
            "cannot get text from response {}\n{:?}",
            error.to_string(),
            response
        );
        error!(error);
        error
    })
}

fn parse_json<T>(json_str: &str) -> Result<T, String>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    // we store the result in a type annotated variable so that serde_json deserialized a T and not a Result<T, String>
    let events: T = serde_json::from_str(json_str).map_err(|error| {
        let error = format!(
            "could not parse json response: {}\n{}",
            error.to_string(),
            json_str
        );
        error!(error);
        error
    })?;
    Ok(events)
}

async fn get_json<T>(url: &str) -> Result<T, String>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    let response = get(url).await.and_then(check_is_ok)?;
    parse_json(&text(response).await?)
}

async fn get_events() -> Result<Vec<Event>, String> {
    get_json::<Vec<Event>>("/api/events").await
}

fn request_events(orders: &mut impl Orders<Msg>) {
    log!("get all events");
    orders.perform_cmd(async {
        match get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
            Err(error) => Msg::Error(error),
        }
    });
}

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    request_events(orders);
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

// ------ ------
//    Update
// ------ ------

enum Msg {
    OnGetEventsResponse(Vec<Event>),
    Error(String),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
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
