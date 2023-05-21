use gloo_net::http::Request;
use seed::{error, log};

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

pub async fn get_json<T>(url: &str) -> Result<T, String>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    let response = get(url).await.and_then(check_is_ok)?;
    parse_json(&text(response).await?)
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
