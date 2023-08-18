
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(deny_unknown_fields)]
pub struct EventEnvSerializer {
    pub channel: Option<String>,
    pub id: Option<String>,
    pub raw: Option<String>,
    pub rawvalue: Option<String>,
    pub timestamp: Option<String>,
    pub value: Option<String>,
}