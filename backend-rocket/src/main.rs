use async_trait::async_trait;
use backend::Backend;
use common::Event;
use rocket::{
    data::Outcome,
    data::{FromData, ToByteUnit},
    fs::NamedFile,
    http::Status,
    request::FromRequest,
    response::status::NotFound,
    Data, Request,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[macro_use]
extern crate rocket;

struct State {
    backend: Backend,
}

impl State {
    fn new() -> State {
        // TODO(hard-coded): get events from database
        State {
            backend: Backend::new(),
        }
    }
}

#[get("/<_url..>")]
async fn index(_url: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend-seed/index.html"))
        .await
        .ok()
}

#[get("/pkg/package.js")]
async fn package_js() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend-seed/pkg/package.js"))
        .await
        .ok()
}

#[get("/pkg/package_bg.wasm")]
async fn package_wasm() -> Option<NamedFile> {
    NamedFile::open(Path::new("../frontend-seed/pkg/package_bg.wasm"))
        .await
        .ok()
}

#[get("/api/events")]
fn events(state: &rocket::State<State>) -> String {
    serde_json::to_string(&state.backend.get_events())
        .unwrap_or("could not serialize events".into())
}

#[get("/api/event/<id_str>")]
fn event(id_str: String, state: &rocket::State<State>) -> Option<String> {
    state
        .backend
        .get_event(&id_str)
        .map(|event| serde_json::to_string(&event).unwrap_or("could not serialize event".into()))
}

struct User {
    name: String,
}

#[crate::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let user_tokens: HashMap<String, String> =
            HashMap::from([("valid_token".to_owned(), "valid_user".to_owned())]);
        match request.headers().get_one("Authorization") {
            Some(authorization) => {
                let parts: Vec<&str> = authorization.trim().split(' ').collect();
                if parts.len() != 2 || parts[0] != "Bearer" {
                    rocket::outcome::Outcome::Failure((
                        Status::BadRequest,
                        "invalid authorization header".to_owned(),
                    ))
                } else {
                    let token = parts[1];
                    match user_tokens.get(token) {
                        Some(username) => rocket::outcome::Outcome::Success(User {
                            name: username.to_string(),
                        }),
                        None => rocket::outcome::Outcome::Failure((
                            Status::BadRequest,
                            "invalid token".to_owned(),
                        )),
                    }
                }
            }
            None => rocket::outcome::Outcome::Failure((
                Status::BadRequest,
                "missing authorization header".to_owned(),
            )),
        }
    }
}

#[put("/api/join/<id_str>")]
fn join_event(
    id_str: String,
    state: &rocket::State<State>,
    user: User,
) -> Result<(), NotFound<String>> {
    state.backend.join_event(&id_str, &user.name)
}

struct EventData {
    event: Event,
}

#[async_trait]
impl<'r> FromData<'r> for EventData {
    async fn from_data(_: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self, Self::Error> {
        match data.open(256.bytes()).into_string().await {
            Ok(json_str) => match serde_json::from_str(json_str.as_str()) {
                Ok(event) => Outcome::Success(EventData { event: event }),
                Err(error) => Outcome::Failure((Status::BadRequest, error.to_string())),
            },
            Err(error) => Outcome::Failure((Status::BadRequest, error.to_string())),
        }
    }

    type Error = String;
}

#[put("/api/event", format = "application/json", data = "<data>")]
fn publish_event(data: EventData, state: &rocket::State<State>) {
    state.backend.publish_event(data.event)
}

#[launch]
fn rocket() -> _ {
    rocket::build().manage(State::new()).mount(
        "/",
        routes![
            event,
            events,
            index,
            join_event,
            package_js,
            package_wasm,
            publish_event,
        ],
    )
}

#[cfg(test)]
mod test {
    use super::rocket;
    use common::{Event, Id};
    use rocket::http::{ContentType, Status};
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
            Event::new("event_1".to_owned()),
            Event::new("event_2".to_owned()),
            Event::new("event_3".to_owned()),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_an_event_by_id() {
        // given 3 existing events
        // TODO(hard-coded): populate events when the events are retrieved from the database

        // when a user requests /api/event/<id>
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        // get an existing id
        let events: Vec<Event> = serde_json::from_str(
            client
                .get(uri!("/api/events"))
                .dispatch()
                .into_string()
                .unwrap()
                .as_str(),
        )
        .unwrap();
        let event = events[0].clone();
        // actual request under test
        let uri = format!("/api/event/{}", event.id);
        let response = client.get(uri).dispatch();

        // then the server responds with a list containing all 3 events
        assert_eq!(response.status(), Status::Ok);
        let actual: Event = serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
        let expected = event;
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_a_non_existing_event_by_id() {
        // given 3 existing events
        // TODO(hard-coded): populate events when the events are retrieved from the database

        // when a user requests a non-existing event using /api/event/<id>
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let uri = format!("/api/event/{}", common::Id::new_v4());
        let response = client.get(uri).dispatch();

        // then the server responds with 404 not found
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn publish_an_event() {
        let published_event_name = "published_event_name";

        fn has_name(name: &str) -> impl Fn(&Event) -> bool {
            let name_owned = name.to_owned();
            return move |event: &Event| event.name == name_owned;
        }

        let client = Client::tracked(rocket()).expect("valid rocket instance");

        // given an non-existing event
        let response = client.get(uri!("/api/events")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let initial_events: Vec<Event> =
            serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
        assert!(initial_events
            .into_iter()
            .filter(has_name(published_event_name))
            .collect::<Vec<Event>>()
            .is_empty());

        // when a client publishes an event
        let event = Event::new(published_event_name.to_owned());
        let event_json = serde_json::to_string(&event).unwrap();
        let response = client
            .put(uri!("/api/event"))
            .header(ContentType::JSON)
            .body(event_json)
            .dispatch();

        // then the server responds with a success code
        assert_eq!(response.status(), Status::Ok);

        // then the event is added to the events list
        let response = client.get(uri!("/api/events")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let initial_events: Vec<Event> =
            serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
        assert!(!initial_events
            .into_iter()
            .filter(has_name(published_event_name))
            .collect::<Vec<Event>>()
            .is_empty());
    }

    fn publish_event(client: &Client, event: &Event) -> Result<(), String> {
        let event_json = serde_json::to_string(&event).unwrap();
        let response = client
            .put(uri!("/api/event"))
            .header(ContentType::JSON)
            .body(event_json)
            .dispatch();
        if response.status() == Status::Ok {
            Ok(())
        } else {
            Err(format!(
                "cannot publish event, received code {}:\n{}",
                response.status(),
                response.into_string().unwrap()
            ))
        }
    }

    fn get_event(client: &Client, id: Id) -> Result<Event, String> {
        let response = client.get(format!("/api/event/{}", id)).dispatch();
        if response.status() != Status::Ok {
            return Err(format!(
                "cannot publish event, received code {}:\n{}",
                response.status(),
                response.into_string().unwrap()
            ));
        }
        let event: Event = serde_json::from_str(response.into_string().unwrap().as_str()).unwrap();
        Ok(event)
    }

    fn join_event(client: &Client, id: Id, token: &str) -> Result<(), String> {
        let response = client
            .put(format!("/api/join/{}", id))
            .header(rocket::http::Header {
                name: "authorization".into(),
                value: format!("Bearer {}", token).into(),
            })
            .dispatch();
        if response.status() == Status::Ok {
            Ok(())
        } else {
            Err(format!(
                "cannot join event, received code {}:\n{}",
                response.status(),
                response.into_string().unwrap()
            ))
        }
    }

    #[test]
    fn joining_an_event() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        // given an event
        let event = Event::new("some_event".to_owned());
        assert_eq!(publish_event(&client, &event), Ok(()));

        // when a logged-in user requests to join the event
        let response = client
            .put(format!("/api/join/{}", event.id))
            .header(rocket::http::Header {
                name: "authorization".into(),
                value: "Bearer valid_token".into(),
            })
            .dispatch();

        // then the server responds with a success code
        assert_eq!(response.status(), Status::Ok);

        // then the user is added to the list of participants in the event
        let updated_event = get_event(&client, event.id).unwrap();
        assert!(updated_event
            .participants
            .contains(&"valid_user".to_owned()));
    }

    #[test]
    fn joining_a_non_existing_event() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        // given a non-existing event
        let id = Id::new_v4();
        assert!(get_event(&client, id).is_err());

        // when a logged-in user requests to join the event
        let response = client
            .put(format!("/api/join/{}", id))
            .header(rocket::http::Header {
                name: "authorization".into(),
                value: "Bearer valid_token".into(),
            })
            .dispatch();

        // then the server responds with a not found code
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn joining_an_event_twice() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        // given an event that a user already joined
        let event = Event::new("some_event".to_owned());
        assert_eq!(publish_event(&client, &event), Ok(()));
        assert_eq!(join_event(&client, event.id, "valid_token"), Ok(()));

        // when a logged-in user requests to join the event
        let response = client
            .put(format!("/api/join/{}", event.id))
            .header(rocket::http::Header {
                name: "authorization".into(),
                value: "Bearer valid_token".into(),
            })
            .dispatch();

        // then the server responds with a success code
        assert_eq!(response.status(), Status::Ok);

        // then the user is not added again to the list of participants in the event
        let updated_event = get_event(&client, event.id).unwrap();
        assert_eq!(updated_event.participants.len(), 1);
        assert!(updated_event
            .participants
            .contains(&"valid_user".to_owned()));
    }
}
