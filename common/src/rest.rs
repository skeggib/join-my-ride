use crate::json::parse_json;
use gloo_net::http::Request;

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
