use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Person {
    pub name: String,
    #[serde(default)]
    pub birth: Option<String>,
    #[serde(default)]
    pub death: Option<String>,
    #[serde(default)]
    pub spouse: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub children: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub root: String,
    pub people: HashMap<String, Person>,
}

impl Database {
    pub fn get(&self, id: &str) -> Option<&Person> {
        self.people.get(id)
    }

    pub fn format_dates(&self, id: &str) -> String {
        let Some(p) = self.get(id) else { return String::new() };
        match (&p.birth, &p.death) {
            (Some(b), Some(d)) => format!("{} – {}", b, d),
            (Some(b), None) => format!("b. {}", b),
            (None, Some(d)) => format!("d. {}", d),
            (None, None) => String::new(),
        }
    }
}
