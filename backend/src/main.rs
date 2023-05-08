#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/api/events")]
fn events() -> &'static str {
    "[{\"name\": \"event_1\"}, {\"name\": \"event_2\"}, {\"name\": \"event_3\"}]"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, events])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn list_all_events() {
        // given 3 existing events
        // TODO: populate events

        // when a user requests /api/events
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/api/events")).dispatch();

        // then the server responds with a list containing all 3 events
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string(),
            Some(
                "[{\"name\": \"event_1\"}, {\"name\": \"event_2\"}, {\"name\": \"event_3\"}]"
                    .into()
            )
        ); // TODO: deserialize JSON
    }
}
