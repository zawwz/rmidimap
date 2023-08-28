use super::EventEnvSerializer;

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(deny_unknown_fields)]
pub struct RunConfigSerializer {
    pub args: Option<Vec<String>>,
    pub cmd:  Option<String>,
    pub envconf: Option<EventEnvSerializer>,
    pub detach: Option<bool>,
}
