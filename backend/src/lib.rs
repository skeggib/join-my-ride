use common::Event;
use rocket::response::status::NotFound;
use std::{ops::DerefMut, str::FromStr, sync::Mutex};

pub struct Backend {
    // TODO(hard-coded): get events from database
    events: Mutex<Vec<Event>>,
}

impl Backend {
    pub fn new() -> Backend {
        Backend {
            events: Mutex::new(vec![
                Event::new("event_1".to_owned()),
                Event::new("event_2".to_owned()),
                Event::new("event_3".to_owned()),
            ]),
        }
    }

    pub fn get_events(self: &Self) -> Vec<Event> {
        match self.events.lock() {
            Ok(events) => events.clone(),
            Err(_) => vec![],
        }
    }

    pub fn get_event(self: &Self, id_str: &str) -> Option<Event> {
        let id = common::Id::from_str(&id_str).ok()?;
        let all_events = self.events.lock().ok()?;
        let matching_events: Vec<&Event> =
            all_events.iter().filter(|event| event.id == id).collect();
        if matching_events.is_empty() {
            None
        } else {
            Some(matching_events[0].clone())
        }
    }

    pub fn join_event(
        self: &Self,
        id_str: &str,
        user_name: &str,
    ) -> Result<(), NotFound<String>> {
        let id =
            common::Id::from_str(&id_str).map_err(|err| NotFound::<String>(err.to_string()))?;
        match self.events.lock() {
            Ok(mut guard) => {
                let all_events = guard.deref_mut();
                let mut found = false;
                for i in 1..all_events.len() {
                    if all_events[i].id == id {
                        found = true;
                        all_events[i].participants.insert(user_name.into());
                    }
                }
                if found {
                    Ok(())
                } else {
                    Err(NotFound::<String>("event not found".to_owned()))
                }
            }
            Err(err) => Err(NotFound::<String>(err.to_string())),
        }
    }

    pub fn publish_event(self: &Self, event: Event) {
        match self.events.lock() {
            Ok(mut guard) => guard.deref_mut().push(event),
            Err(_) => {}
        }
    }
}
