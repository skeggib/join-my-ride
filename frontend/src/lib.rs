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

async fn put_json<T>(url: &str, value: &T) -> Result<(), String>
where
    T: serde::Serialize,
{
    let request = Request::new(url)
        .method(gloo_net::http::Method::PUT)
        .json(value)
        .map_err(|error| error.to_string())?;
    request.send().await.map_err(|error| error.to_string())?;
    Ok(())
}

async fn get_events() -> Result<Vec<Event>, String> {
    get_json::<Vec<Event>>("/api/events").await
}

async fn put_event(name: String) -> Result<(), String> {
    let event = Event { name: name };
    put_json("/api/event", &event).await
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

fn clear_publish_event_form(model: &Model) -> State {
    match &model.state {
        State::Loaded(loaded_state) => {
            let mut new_state = loaded_state.clone();
            new_state.publish_event_name = "".to_owned();
            State::Loaded(new_state)
        }
        _ => model.state.clone(),
    }
}

fn publish_event(state: &LoadedState, orders: &mut impl Orders<Msg>) {
    log!("publish event");
    let name = state.publish_event_name.clone();
    orders.perform_cmd(async move {
        match put_event(name).await {
            Ok(_) => {}
            Err(error) => error!(error),
        }
        ()
    });
}

fn publish_event_click(model: &Model, orders: &mut impl Orders<Msg>) -> State {
    match &model.state {
        State::Loaded(loaded_state) => {
            publish_event(loaded_state, orders);
            request_events(orders);
            clear_publish_event_form(model)
        }
        _ => model.state.clone(),
    }
}

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    request_events(orders);
    Model {
        state: State::Loading,
    }
}

// ------ ------
//     Model
// ------ ------

type Events = Vec<Event>;
type ErrorMessage = String;

#[derive(Clone)]
enum State {
    Loading,
    Loaded(LoadedState),
    Failed(ErrorMessage),
}

#[derive(Clone)]
struct LoadedState {
    events: Events,
    publish_event_name: String,
}

struct Model {
    state: State,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    OnGetEventsResponse(Vec<Event>),
    PublishEventNameChange(String),
    PublishEventClick,
    Error(String),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    model.state = match msg {
        Msg::OnGetEventsResponse(events) => match &model.state {
            State::Loading => State::Loaded(LoadedState {
                events: events,
                publish_event_name: "".to_owned(),
            }),
            State::Loaded(loaded_state) => {
                let mut new_state = loaded_state.clone();
                new_state.events = events;
                State::Loaded(new_state)
            }
            _ => model.state.clone(),
        },
        Msg::Error(error) => State::Failed(error),
        Msg::PublishEventNameChange(new_name) => match &model.state {
            State::Loaded(loaded_state) => {
                let mut new_state = loaded_state.clone();
                new_state.publish_event_name = new_name;
                State::Loaded(new_state)
            }
            _ => model.state.clone(),
        },
        Msg::PublishEventClick => publish_event_click(model, orders),
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    match &model.state {
        State::Loading => div!["loading..."],
        State::Loaded(loaded_state) => {
            let event_divs: Vec<Node<Msg>> = loaded_state
                .events
                .iter()
                .map(|event| div![event.name.clone()])
                .collect();
            div![
                div![event_divs],
                input![
                    attrs![At::Value=>loaded_state.publish_event_name.clone()],
                    input_ev(Ev::Input, |value| { Msg::PublishEventNameChange(value) })
                ],
                button!["publish event", ev(Ev::Click, |_| Msg::PublishEventClick)]
            ]
        }
        State::Failed(error) => div![error],
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
