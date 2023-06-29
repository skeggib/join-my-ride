pub fn parse_json<T>(json_str: &str) -> Result<T, String>
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
        error
    })?;
    Ok(events)
}
