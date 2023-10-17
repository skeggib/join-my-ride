use crate::{json::parse_json, api::BackendApi, Event, Id};
use async_trait::async_trait;
use gloo_net::http::Request;

pub struct RestBackend {}

#[async_trait(?Send)]
impl BackendApi for RestBackend {
    async fn get_events(self: &Self) -> Result<Vec<Event>, String> {
        get_json::<Vec<Event>>("/api/events").await
    }
    async fn get_event(self: &Self, id: Id) -> Result<Event, String> {
        get_json::<Event>(&format!("/api/event/{}", id)).await
    }
    async fn publish_event(self: &Self, name: String) -> Result<(), String> {
        let event = Event::new(name);
        put_json("/api/event", &event).await
    }
    async fn join_event(self: &Self, id: Id) -> Result<(), String> {
        put(&format!("/api/join/{}", id)).await
    }
}

fn check_is_ok(response: gloo_net::http::Response) -> Result<gloo_net::http::Response, String> {
    if response.ok() {
        Ok(response)
    } else {
        let error = format!("server responded with code {}", response.status());
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
        error
    })
}

async fn get(url: &str) -> Result<gloo_net::http::Response, String> {
    let request = Request::new(url);
    request.send().await.map_err(|error| {
        let error = format!("could not send request {}", error.to_string());
        error
    })
}

pub async fn get_json<T>(url: &str) -> Result<T, String>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    let response = get(url).await.and_then(check_is_ok)?;
    parse_json(&text(response).await?)
}

pub async fn put(url: &str) -> Result<(), String> {
    let request = Request::new(url)
        .method(gloo_net::http::Method::PUT)
        .header("Authorization", "Bearer valid_token");
    request
        .send()
        .await
        .map_err(|error| error.to_string())
        .and_then(check_is_ok)?;
    Ok(())
}

pub async fn put_json<T>(url: &str, value: &T) -> Result<(), String>
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
