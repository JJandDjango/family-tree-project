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
    pub place: Option<String>,
    #[serde(default)]
    pub deceased: bool,
    #[serde(default)]
    pub anonymized: bool,
    #[serde(default)]
    pub spouse: Option<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DatabaseRaw {
    pub focus: String,
    pub people: HashMap<String, Person>,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub focus: String,
    pub people: HashMap<String, Person>,
    children: HashMap<String, Vec<String>>,
}

impl Database {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        let raw: DatabaseRaw = serde_json::from_str(json)?;
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        let mut ids: Vec<&String> = raw.people.keys().collect();
        ids.sort();
        for id in ids {
            if let Some(person) = raw.people.get(id) {
                for parent_id in &person.parents {
                    children
                        .entry(parent_id.clone())
                        .or_default()
                        .push(id.clone());
                }
            }
        }
        Ok(Self {
            focus: raw.focus,
            people: raw.people,
            children,
        })
    }

    pub fn get(&self, id: &str) -> Option<&Person> {
        self.people.get(id)
    }

    pub fn children_of(&self, id: &str) -> &[String] {
        self.children
            .get(id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn format_dates(&self, id: &str) -> String {
        let Some(p) = self.get(id) else {
            return String::new();
        };
        if p.anonymized {
            return String::new();
        }
        match (&p.birth, &p.death) {
            (Some(b), Some(d)) => format!("{} – {}", b, d),
            (Some(b), None) if p.deceased => format!("b. {} (d.)", b),
            (Some(b), None) => format!("b. {}", b),
            (None, Some(d)) => format!("d. {}", d),
            (None, None) if p.deceased => "(deceased)".to_string(),
            (None, None) => String::new(),
        }
    }

    /// Returns the union of children across the given parent ids, deduplicated, in stable order.
    pub fn children_of_any(&self, parent_ids: &[String]) -> Vec<String> {
        use std::collections::HashSet;
        let mut seen: HashSet<&str> = HashSet::new();
        let mut out: Vec<String> = Vec::new();
        for pid in parent_ids {
            for cid in self.children_of(pid) {
                if seen.insert(cid.as_str()) {
                    out.push(cid.clone());
                }
            }
        }
        out
    }
}
