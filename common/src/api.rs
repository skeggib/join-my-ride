use crate::{Event, Id};
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait BackendApi {
    async fn get_events(self: &Self) -> Result<Vec<Event>, String>;
    async fn get_event(self: &Self, id: Id) -> Result<Event, String>;
    async fn publish_event(self: &Self, name: String) -> Result<(), String>;
    async fn join_event(self: &Self, id: Id) -> Result<(), String>;
}
