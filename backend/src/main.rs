use std::path::{PathBuf, Path};

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
    NamedFile::open(Path::new("../frontend/").join(file)).await.ok()
}

#[get("/api/events")]
fn events() -> &'static str {
    // TODO(hard-coded): get from database
    // TODO(wip): serialize objects to JSON
    "[{\"name\": \"event_1\"}, {\"name\": \"event_2\"}, {\"name\": \"event_3\"}]"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, files, events])
}

#[cfg(test)]
mod test {
    use super::rocket;
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
        assert_eq!(
            response.into_string(),
            Some(
                "[{\"name\": \"event_1\"}, {\"name\": \"event_2\"}, {\"name\": \"event_3\"}]" // TODO(wip): deserialize JSON
                    .into()
            )
        );
    }
}
