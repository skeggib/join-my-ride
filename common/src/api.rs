use crate::{
    rest::{get_json, put, put_json},
    Event, Id,
};

pub async fn get_events() -> Result<Vec<Event>, String> {
    get_json::<Vec<Event>>("/api/events").await
}

pub async fn get_event(id: Id) -> Result<Event, String> {
    get_json::<Event>(&format!("/api/event/{}", id)).await
}

pub async fn publish_event(name: String) -> Result<(), String> {
    let event = Event::new(name);
    put_json("/api/event", &event).await
}

pub async fn join_event(id: Id) -> Result<(), String> {
    put(&format!("/api/join/{}", id)).await
}
