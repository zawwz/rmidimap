use std::collections::HashMap;

use crate::config::{EventConfig,DeviceConfig};
use crate::event::{EventType,Event};
use crate::util::SmartSet;

use std::collections::BTreeSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref NOTE_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from((0..=127).collect::<BTreeSet<u8>>()),
    };
    static ref NOTE_DEFAULT_MAP_SIZE: usize = NOTE_DEFAULT_MAP.len();
    static ref NULL_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from([0]),
    };
    static ref NULL_DEFAULT_MAP_SIZE: usize = NULL_DEFAULT_MAP.len();
    static ref CHANNEL_DEFAULT_MAP: SmartSet<u8> = SmartSet {
        set: BTreeSet::from((0..=15).collect::<BTreeSet<u8>>()),
    };
    static ref CHANNEL_DEFAULT_MAP_SIZE: usize = CHANNEL_DEFAULT_MAP.len();
}

#[derive(Debug,Default)]
pub struct EventMap<'a> {
    //TODO: vec support
    pub map: HashMap<u32, Vec<&'a EventConfig>>,
}

fn event_to_key(r#type: EventType, channel: u8, id: u8) -> u32 {
    (r#type as u32)*256*256 + (channel as u32)*256 + (id as u32)
}

pub fn count_events(events: &[EventConfig]) -> usize {
    events.iter().map(|x| {
        let nchannel = match x.r#type.has_channel() {
            true  => x.channel.as_ref().map_or(*CHANNEL_DEFAULT_MAP_SIZE, |x| x.len()),
            false => *CHANNEL_DEFAULT_MAP_SIZE,
        };
        let nid = match x.r#type.has_id() {
            true  => x.id.as_ref().map_or(*NOTE_DEFAULT_MAP_SIZE, |x| x.len()),
            false => *NULL_DEFAULT_MAP_SIZE,
        };
        nchannel * nid
    }).sum()
}

impl<'a> EventMap<'a> {
    pub fn add_events(&mut self, events: &'a [EventConfig]) {
        for event in events {
            for &channel in match event.r#type.has_id() {
                true  => event.channel.as_ref().unwrap_or(&CHANNEL_DEFAULT_MAP),
                false => &CHANNEL_DEFAULT_MAP,
            } {
                for &id in 
                    match event.r#type.has_id() {
                    true  => event.id.as_ref().unwrap_or(&NOTE_DEFAULT_MAP),
                    false => &NULL_DEFAULT_MAP,
                } {
                    let key = event_to_key(event.r#type, channel, id);
                    if let Some(v) = self.map.get_mut(&key) {
                        v.push(event);
                    }
                    else {
                        self.map.insert(key, Vec::from([event]));
                    }
                }
            }
        }
    }

    pub fn run_event(&self, event: &Event) -> Result<(), std::io::Error > {
        let key = event_to_key(event.r#type, event.channel, event.id);
        if let Some(v) = self.map.get(&key) {
            for ev in v {
                for r in &ev.run {
                    r.run(event.gen_env())?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> From<&'a [EventConfig]> for EventMap<'a> {
    fn from(events: &'a [EventConfig]) -> Self {
        // init hashmap with size for optimizing
        let size = count_events(events);
        let mut ret = EventMap { map: HashMap::with_capacity(size) };
        // insert references
        ret.add_events(events);
        ret
    }

}

impl<'a> From<&'a DeviceConfig> for EventMap<'a> {
    fn from(device: &'a DeviceConfig) -> Self {
        // init hashmap with size for optimizing
        let size = count_events(device.events.as_ref().map(|x| &x[..]).unwrap_or(&[]));
        //let size = events.iter().map(|x| x.channels.len()*x.ids.len() ).sum();
        let mut ret = EventMap { map: HashMap::with_capacity(size) };
        // insert references
        if let Some(x) = device.events.as_ref() { 
            ret.add_events(x);
        }
        ret
    }

}
