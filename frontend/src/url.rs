use std::{str::FromStr, fmt::Display};

/// Replaces the seed Url implementation which cannot be used in non-wasm targets
#[derive(Clone, Debug)]
pub struct Url {
    path: Vec<String>,
    next_path_part_index: usize,
}

impl Url {
    pub fn new() -> Url {
        Url {
            path: vec![],
            next_path_part_index: 0,
        }
    }

    pub fn next_path_part(self: &mut Self) -> Option<&str> {
        if self.next_path_part_index < self.path.len() {
            let next_path_part_index = self.next_path_part_index;
            self.next_path_part_index += 1;
            Some(&self.path[next_path_part_index])
        } else {
            None
        }
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.join("/"))
    }
}

impl FromStr for Url {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Url {
            path: s.split("/").map(str::to_owned).filter(|part| !part.is_empty()).collect(),
            next_path_part_index: 0,
        })
    }
}
