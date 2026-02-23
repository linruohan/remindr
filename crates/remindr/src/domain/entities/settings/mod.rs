use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DbContext {
    Local(LocalDatabase),
    Remote(RemoteDatabase),
    #[serde(other)]
    Unknown,
}

impl DbContext {
    pub fn parse(value: Value) -> DbContext {
        from_value::<DbContext>(value).unwrap_or(DbContext::Unknown)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalDatabase {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteDatabase {
    pub name: String,
    pub url: String,
}
