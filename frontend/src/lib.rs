// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use std::future::Future;

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
    perform_cmd(orders, async {
        match get_events().await {
            Ok(events) => Msg::EventList(MsgEventList::OnGetEventsResponse(events)),
            Err(error) => Msg::Error(error),
        }
    });
}

// wrapper around orders.perform_cmd to make the cmd strongly typed
fn perform_cmd(orders: &mut impl Orders<Msg>, cmd: impl Future<Output = Msg> + 'static) {
    orders.perform_cmd(cmd);
}

fn publish_event(contents: &EventPublicationFormContents, orders: &mut impl Orders<Msg>) {
    log!("publish event");
    let name = contents.event_name.clone();
    perform_cmd(orders, async move {
        match put_event(name).await {
            Ok(_) => Msg::EventPublicationForm(MsgEventPublicationForm::OnEventPublished),
            Err(error) => {
                error!(error);
                todo!()
            }
        }
    });
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
    event_list: Events,
    event_publication_form: EventPublicationForm,
}

#[derive(Clone)]
enum EventPublicationForm {
    Empty,
    Typing(EventPublicationFormContents),
    Publishing(EventPublicationFormContents),
    Invalid((Option<EventPublicationFormContents>, String)),
}

#[derive(Clone)]
struct EventPublicationFormContents {
    event_name: String,
}

struct Model {
    state: State,
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
enum Msg {
    EventList(MsgEventList),
    EventPublicationForm(MsgEventPublicationForm),
    Error(String),
}

#[derive(Clone)]
enum MsgEventList {
    OnGetEventsResponse(Vec<Event>),
}

#[derive(Clone)]
enum MsgEventPublicationForm {
    PublishEventNameChange(String),
    PublishEventClick,
    OnEventPublished,
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
            MsgEventList::OnGetEventsResponse(events) => State::Loaded(LoadedState {
                event_list: events,
                event_publication_form: EventPublicationForm::Empty,
            }),
        },
        Msg::EventPublicationForm(_) => {
            error!("received a publication form message while loading");
            State::Loading
        }
        Msg::Error(error) => State::Failed(error),
    }
}

fn update_state_loaded(
    msg: Msg,
    loaded: &LoadedState,
    orders: &mut impl Orders<Msg>,
) -> Result<LoadedState, String> {
    match msg {
        Msg::EventPublicationForm(MsgEventPublicationForm::OnEventPublished) => {
            request_events(orders);
        }
        _ => {}
    };
    Ok(LoadedState {
        event_list: match msg.clone() {
            Msg::EventList(msg_event_list) => update_event_list(msg_event_list)?,
            _ => loaded.event_list.clone(),
        },
        event_publication_form: match msg {
            Msg::EventPublicationForm(msg_event_publication_form) => update_event_publication_form(
                msg_event_publication_form,
                &loaded.event_publication_form,
                orders,
            )?,
            _ => loaded.event_publication_form.clone(),
        },
    })
}

fn update_event_list(msg: MsgEventList) -> Result<Events, String> {
    Ok(match msg {
        MsgEventList::OnGetEventsResponse(events) => events,
    })
}

fn update_event_publication_form(
    msg: MsgEventPublicationForm,
    form: &EventPublicationForm,
    orders: &mut impl Orders<Msg>,
) -> Result<EventPublicationForm, String> {
    match form {
        EventPublicationForm::Empty => update_event_publication_form_empty(msg),
        EventPublicationForm::Typing(contents) => {
            update_event_publication_form_typing(msg, contents, orders)
        }
        EventPublicationForm::Publishing(contents) => {
            update_event_publication_form_publishing(msg, contents, orders)
        }
        EventPublicationForm::Invalid((contents, _)) => {
            update_event_publication_form_invalid(msg, contents)
        }
    }
}

fn update_event_publication_form_empty(
    msg: MsgEventPublicationForm,
) -> Result<EventPublicationForm, String> {
    match msg {
        MsgEventPublicationForm::PublishEventNameChange(name) => {
            Ok(EventPublicationForm::Typing(EventPublicationFormContents {
                event_name: name,
            }))
        }
        MsgEventPublicationForm::PublishEventClick => Ok(EventPublicationForm::Invalid((
            None,
            "The name is required".into(),
        ))),
        MsgEventPublicationForm::OnEventPublished => {
            error!("should not happen");
            Ok(EventPublicationForm::Empty)
        }
    }
}

fn update_event_publication_form_typing(
    msg: MsgEventPublicationForm,
    contents: &EventPublicationFormContents,
    orders: &mut impl Orders<Msg>,
) -> Result<EventPublicationForm, String> {
    match msg {
        MsgEventPublicationForm::PublishEventNameChange(name) => {
            let mut new_contents = contents.clone();
            new_contents.event_name = name;
            Ok(EventPublicationForm::Typing(new_contents))
        }
        MsgEventPublicationForm::PublishEventClick => {
            if contents.event_name.is_empty() {
                Ok(EventPublicationForm::Invalid((
                    Some(contents.clone()),
                    "The name is required".into(),
                )))
            } else {
                publish_event(contents, orders);
                Ok(EventPublicationForm::Publishing(contents.clone()))
            }
        }
        MsgEventPublicationForm::OnEventPublished => {
            error!("should not happen");
            Ok(EventPublicationForm::Empty)
        }
    }
}

fn update_event_publication_form_publishing(
    msg: MsgEventPublicationForm,
    contents: &EventPublicationFormContents,
    orders: &mut impl Orders<Msg>,
) -> Result<EventPublicationForm, String> {
    match msg {
        MsgEventPublicationForm::PublishEventNameChange(name) => {
            let mut new_contents = contents.clone();
            new_contents.event_name = name;
            Ok(EventPublicationForm::Publishing(new_contents))
        }
        MsgEventPublicationForm::PublishEventClick => {
            publish_event(contents, orders);
            Ok(EventPublicationForm::Publishing(contents.clone()))
        }
        MsgEventPublicationForm::OnEventPublished => Ok(EventPublicationForm::Empty),
    }
}

fn update_event_publication_form_invalid(
    msg: MsgEventPublicationForm,
    maybe_contents: &Option<EventPublicationFormContents>,
) -> Result<EventPublicationForm, String> {
    match maybe_contents {
        Some(contents) => match msg {
            MsgEventPublicationForm::PublishEventNameChange(name) => {
                let mut new_contents = contents.clone();
                new_contents.event_name = name;
                Ok(EventPublicationForm::Typing(new_contents))
            }
            MsgEventPublicationForm::PublishEventClick => {
                Ok(EventPublicationForm::Typing(contents.clone()))
            }
            MsgEventPublicationForm::OnEventPublished => {
                error!("should not happen");
                Ok(EventPublicationForm::Empty)
            }
        },
        None => Ok(EventPublicationForm::Empty),
    }
}

// ------ ------
//     View
// ------ ------

fn view_event_list(state: &LoadedState) -> Node<Msg> {
    let event_divs: Vec<Node<Msg>> = state
        .event_list
        .iter()
        .map(|event| div![event.name.clone()])
        .collect();
    div![event_divs]
}

fn view_event_name_input(state: &LoadedState) -> Node<Msg> {
    let value = match &state.event_publication_form {
        EventPublicationForm::Empty => "".into(),
        EventPublicationForm::Typing(contents) => contents.event_name.clone(),
        EventPublicationForm::Publishing(contents) => contents.event_name.clone(),
        EventPublicationForm::Invalid((maybe_contents, _)) => match maybe_contents {
            Some(contents) => contents.event_name.clone(),
            None => "".into(),
        },
    };
    let input = input![
        attrs![At::Value => value],
        input_ev(Ev::Input, |value| {
            Msg::EventPublicationForm(MsgEventPublicationForm::PublishEventNameChange(value))
        })
    ];
    match &state.event_publication_form {
        EventPublicationForm::Invalid((_, error)) => div![input, error],
        _ => div![input],
    }
}

fn view(model: &Model) -> Node<Msg> {
    match &model.state {
        State::Loading => div!["loading..."],
        State::Loaded(loaded_state) => {
            let is_form_ready_for_publishing = match loaded_state.event_publication_form {
                EventPublicationForm::Empty => false,
                EventPublicationForm::Typing(_) => true,
                EventPublicationForm::Publishing(_) => false,
                EventPublicationForm::Invalid(_) => false,
            };
            div![
                view_event_list(loaded_state),
                view_event_name_input(loaded_state),
                button![
                    "publish event",
                    attrs![At::Disabled => (!is_form_ready_for_publishing).as_at_value()],
                    ev(Ev::Click, |_| Msg::EventPublicationForm(
                        MsgEventPublicationForm::PublishEventClick
                    ))
                ]
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
