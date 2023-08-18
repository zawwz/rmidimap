use super::RunConfig;
use crate::event::{Event,EventType};
use crate::util::{self, SmartSet, Range, Remapper};

use super::serializer::EventConfigSerializer;

use std::collections::BTreeSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref ID_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from((0..=127).collect::<BTreeSet<u8>>()),
    };
    static ref NULL_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from([0]),
    };
    static ref CHANNEL_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from((0..=15).collect::<BTreeSet<u8>>()),
    };
    static ref TRIGGER_NOTE_DEFAULT_MAP: SmartSet<u16> = SmartSet {
        set: BTreeSet::from((1..=127).collect::<BTreeSet<u16>>()),
    };
    static ref TRIGGER_U8_DEFAULT_MAP: SmartSet<u16> = SmartSet {
        set: BTreeSet::from((0..=127).collect::<BTreeSet<u16>>()),
    };
    static ref TRIGGER_U16_DEFAULT_MAP: SmartSet<u16> = SmartSet {
        set: BTreeSet::from((0..=65535).collect::<BTreeSet<u16>>()),
    };
    static ref TRIGGER_NULL_DEFAULT_MAP: SmartSet<u16> = SmartSet {
        set: BTreeSet::from([0]),
    };
}

#[derive(Debug,Clone)]
pub struct EventConfig {
    pub run: Vec<RunConfig>,
    pub r#type: EventType,
    pub channel: SmartSet<u8>,
    pub id: SmartSet<u8>,
    pub remap: Option<Remapper<f64>>,
    pub float: bool,
    pub value: Option<SmartSet<u16>>,
}

impl EventConfig {
    pub fn match_value(&self, event: &Event) -> bool {
        match &self.value {
            Some(v) =>  v.set.contains(&event.value),
            None => true,
        }
    }
}

impl TryFrom<EventConfigSerializer> for EventConfig {
    type Error = crate::Error;
    fn try_from(v: EventConfigSerializer) -> Result<Self, Self::Error> {
        let r = EventConfig {
            run: util::map_tryfrom(v.run)?,
            r#type: v.r#type,
            channel: match v.r#type.has_channel() {
                true  => v.channel.unwrap_or_else(|| CHANNEL_DEFAULT_MAP.clone()),
                false => NULL_DEFAULT_MAP.clone(),
            },
            id: match v.r#type.has_id() {
                true  => v.id.unwrap_or_else(|| ID_DEFAULT_MAP.clone()),
                false => NULL_DEFAULT_MAP.clone(),
            },
            remap: v.remap.map(|x| Remapper::new(Range::new(v.r#type.min_value() as f64, v.r#type.max_value() as f64), x )),
            float: v.float.unwrap_or(false),
            value: v.value,
        };
        if let Some(remap) = &r.remap {
            let range = remap.src();
            if range.start() < i64::MIN as f64 { return Err(Self::Error::RemapTooLow(range.start())) }
            if range.end() > i64::MAX as f64  { return Err(Self::Error::RemapTooBig(range.end())) }
        }
        Ok(r)
    }
}
