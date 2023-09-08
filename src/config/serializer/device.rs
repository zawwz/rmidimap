use std::time::Duration;

use super::{RunConfigSerializer,EventConfigSerializer};

use duration_str::deserialize_duration;
use serde::Deserialize;

#[derive(Debug,Clone,Deserialize)]
#[serde(untagged)]
pub enum DurationWrapper {
    #[serde(deserialize_with = "deserialize_duration")]
    Some(Duration),
}

impl DurationWrapper {
    pub fn unwrap(self) -> Duration
    {
        match self {
            DurationWrapper::Some(v) => v,
        }
    }
}

#[derive(Deserialize,Debug,Clone)]
#[serde(deny_unknown_fields)]
pub struct DeviceConfigSerializer {
    pub name: Option<String>,
    pub regex: Option<String>,
    pub addr: Option<String>,
    pub connect: Option<Vec<RunConfigSerializer>>,
    pub disconnect: Option<Vec<RunConfigSerializer>>,
    pub events: Option<Vec<EventConfigSerializer>>,
    pub max_connections: Option<u32>,
    pub queue_length: Option<usize>,
    pub interval: Option<DurationWrapper>,
    pub log_events: Option<bool>,
}
