use std::collections::HashMap;

use crate::config::{EventConfig,DeviceConfig};
use crate::event::{EventType,Event};
use crate::Error;

#[derive(Debug,Default)]
pub struct EventMap<'a> {
    pub map: HashMap<u32, Vec<&'a EventConfig>>,
}

fn event_to_key(r#type: EventType, channel: u8, id: u8) -> u32 {
    (r#type as u32)*256*256 + (channel as u32)*256 + (id as u32)
}

pub fn count_events(events: &[EventConfig]) -> usize {
    events.iter().map(|x| {
        let nchannel = x.channel.len();
        let nid = x.id.len();
        nchannel * nid
    }).sum()
}

impl<'a> EventMap<'a> {
    pub fn add_events(&mut self, events: &'a [EventConfig]) {
        for event in events {
            for &channel in &event.channel {
                for &id in &event.id {
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

    pub fn run_event(&self, event: &Event) -> Result<(), Error > {
        let key = event_to_key(event.r#type, event.channel, event.id);
        if let Some(v) = self.map.get(&key) {
            for ev in v {
                if ev.match_value(event) {
                    for r in &ev.run {
                        r.run(event.make_env(ev.remap.as_ref(), ev.float )?.to_map(r.envconf.as_ref()))?;
                    }
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
