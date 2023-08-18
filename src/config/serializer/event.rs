use super::RunConfigSerializer;
use crate::event::EventType;
use crate::util::{SmartSet,Range};

use serde::Deserialize;

#[derive(Deserialize,Debug,Clone)]
#[serde(deny_unknown_fields)]
pub struct EventConfigSerializer {
    pub run: Vec<RunConfigSerializer>,
    pub r#type: EventType,
    pub channel: Option<SmartSet<u8>>,
    pub id: Option<SmartSet<u8>>,
    pub remap: Option<Range<f64>>,
    pub float: Option<bool>,
    pub value: Option<SmartSet<u16>>,
}
