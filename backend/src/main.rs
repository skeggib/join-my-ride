use std::path::{Path, PathBuf};

use common::Event;
use rocket::fs::NamedFile;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    // TODO(security): it is unsafe to allow requesting any file in frontend
    // TODO(security): the file path needs sanitizing and checking for relative paths
    NamedFile::open(Path::new("../frontend/").join(file))
        .await
        .ok()
}

#[get("/api/events")]
fn events() -> String {
    // TODO(hard-coded): get from database
    let events = vec![
        Event {
            name: "event_1".to_owned(),
        },
        Event {
            name: "event_2".to_owned(),
        },
        Event {
            name: "event_3".to_owned(),
        },
    ];
    serde_json::to_string(&events).unwrap()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, files, events])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use common::Event;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn list_all_events() {
        // given 3 existing events
        // TODO(hard-coded): populate events when the events are retrieved from the database

        // when a user requests /api/events
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/api/events")).dispatch();

        // then the server responds with a list containing all 3 events
        assert_eq!(response.status(), Status::Ok);
        let actual: Vec<Event> =
            serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
        let expected = vec![
            Event {
                name: "event_1".to_owned(),
            },
            Event {
                name: "event_2".to_owned(),
            },
            Event {
                name: "event_3".to_owned(),
            },
        ];
        assert_eq!(actual, expected);
    }
}
