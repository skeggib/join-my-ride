use common::Event;
use gloo_net::http::Request;
use seed::{prelude::*, *};
use std::future::Future;

pub type Model = Vec<Event>;

// TODO: move to common code
async fn get(url: &str) -> Result<gloo_net::http::Response, String> {
    let request = Request::new(url);
    log!(format!("send request {:?}", request));
    request.send().await.map_err(|error| {
        let error = format!("could not send request {}", error.to_string());
        error!(error);
        error
    })
}

// TODO: move to common code
fn check_is_ok(response: gloo_net::http::Response) -> Result<gloo_net::http::Response, String> {
    if response.ok() {
        Ok(response)
    } else {
        let error = format!("server responded with code {}", response.status());
        error!(error);
        Err(error)
    }
}

// TODO: move to common code
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

// TODO: move to common code
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

// TODO: move to common code
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

pub fn request_events(orders: &mut impl Orders<Msg>) {
    log!("get all events");
    perform_cmd(orders, async {
        match get_events().await {
            Ok(events) => Msg::OnGetEventsResponse(events),
            Err(error) => Msg::Error(error),
        }
    });
}

// TODO: move to common code
fn perform_cmd(orders: &mut impl Orders<Msg>, cmd: impl Future<Output = Msg> + 'static) {
    orders.perform_cmd(cmd);
}

#[derive(Clone)]
pub enum Msg {
    OnGetEventsResponse(Vec<Event>),
    Error(String),
}

pub fn update(msg: Msg) -> Result<Model, String> {
    match msg {
        Msg::OnGetEventsResponse(events) => Ok(events),
        Msg::Error(error) => Err(error),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let event_divs: Vec<Node<Msg>> = model.iter().map(|event| div![event.name.clone()]).collect();
    div![event_divs]
}
