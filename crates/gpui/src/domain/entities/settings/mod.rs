use serde::{Deserialize, Serialize};
use serde_json::{Value, from_value};

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct LocalDatabase {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoteDatabase {
    pub name: String,
    pub url: String,
}
