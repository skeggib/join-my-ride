use std::collections::HashSet;

use derivative::Derivative;
use serde;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Id = Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Derivative)]
#[derivative(PartialEq)]
pub struct Event {
    #[serde(with = "uuid_codec")]
    #[derivative(PartialEq = "ignore")]
    pub id: Id,
    pub name: String,
    pub participants: HashSet<String>,
}

impl Event {
    pub fn new(name: String) -> Event {
        Event {
            id: Id::new_v4(),
            name: name,
            participants: HashSet::new(),
        }
    }
}

mod uuid_codec {
    use serde::{
        de::{Error, Unexpected},
        Deserialize, Deserializer, Serializer,
    };
    use std::str::FromStr;
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match Uuid::from_str(&value) {
            Ok(uuid) => Ok(uuid),
            Err(_) => Err(Error::invalid_value(Unexpected::Str(&value), &"UUID")),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Event;

    #[test]
    fn serializing_an_event() {
        let event = Event::new("name".to_owned());
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }
}
