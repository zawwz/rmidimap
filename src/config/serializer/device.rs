use super::{RunConfigSerializer,EventConfigSerializer};

use serde::Deserialize;

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
}
