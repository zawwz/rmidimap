use crate::config::RunConfig;
use crate::event::EventType;
use crate::event::Event;
use crate::util::SmartSet;

use serde::Deserialize;

#[derive(Deserialize,Debug,Clone)]
pub struct EventConfig {
    pub run: Vec<RunConfig>,
    pub r#type: EventType,
    pub channel: Option<SmartSet<u8>>,
    pub id: Option<SmartSet<u8>>,
    // pub channels: BTreeSet<u8>,
    // pub ids: BTreeSet<u8>,
    // TODO: rework for value conditions (for pitch) ?
    //values: BTreeSet<u8>,
    //values: Condition,
}

impl EventConfig {
    pub fn matches(&self, event: &Event) -> bool {
        //TODO: value conditions don't exist yet
        true
    }
}
