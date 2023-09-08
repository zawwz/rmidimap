use std::{collections::HashMap, time::SystemTime};
use std::fmt::{Write,Display};

use crate::config::EventEnvMap;
use crate::util::Remapper;
use crate::Error;

use serde::{Serialize,Deserialize};

use lazy_static::lazy_static;

lazy_static! {
    static ref EVENT_ENV_DEFAULT: EventEnvRef<'static> = EventEnvRef {
        channel: "channel",
        id: "id",
        raw: "raw",
        rawvalue: "rawvalue",
        timestamp: "timestamp",
        value: "value",
    };
}

pub fn event_to_key(r#type: EventType, channel: u8, id: u8) -> u32 {
    (r#type as u32)*256*256 + (channel as u32)*256 + (id as u32)
}

#[repr(u8)]
#[derive(Serialize,Deserialize,Debug,Copy,Clone,Default,Display)]
pub enum EventType {
    #[default]
    Unknown                 = 0b0000,
    NoteOff                 = 0b1000,
    NoteOn                  = 0b1001,
    PolyphonicKeyPressure   = 0b1010,
    Controller              = 0b1011,
    ProgramChange           = 0b1100,
    ChannelPressure         = 0b1101,
    PitchBend               = 0b1110,
    System                  = 0b1111,
}

impl EventType {
    pub fn has_id(&self) -> bool {
        !matches!(self, EventType::Unknown | EventType::ChannelPressure | EventType::PitchBend | EventType::System )
    }
    pub fn has_channel(&self) -> bool {
        !matches!(self, EventType::Unknown | EventType::System )
    }
    pub fn min_value(&self) -> i32 {
        match self {
            EventType::NoteOff |
            EventType::NoteOn |
            EventType::Controller
                => 0,
            EventType::PolyphonicKeyPressure |
            EventType::ChannelPressure
                => 127,
            EventType::PitchBend
                => 0,
            _ => 0,
        }
    }
    pub fn max_value(&self) -> i32 {
        match self {
            EventType::NoteOff |
            EventType::NoteOn |
            EventType::Controller
                => 127,
            EventType::PolyphonicKeyPressure |
            EventType::ChannelPressure
                => 127,
            EventType::PitchBend
                => 32767,
            _ => 0,
        }
    }
}

#[derive(Debug,Default)]
pub struct Event<'a> {
    pub r#type: EventType,
    pub channel: u8,
    pub id: u8,
    pub value: u16,
    pub raw: &'a [u8],
    pub timestamp: Option<SystemTime>,
}

#[derive(Debug,Clone,Default)]
pub struct EventBuf {
    pub r#type: EventType,
    pub channel: u8,
    pub id: u8,
    pub value: u16,
    pub raw: Vec<u8>,
    pub timestamp: Option<SystemTime>,
}

pub struct EventEnv {
    pub channel: String,
    pub id: String,
    pub raw: String,
    pub rawvalue: String,
    pub timestamp: String,
    pub value: String,
}

#[derive(Clone,Debug)]
struct EventEnvRef<'a> {
    pub channel: &'a str,
    pub id: &'a str,
    pub raw: &'a str,
    pub rawvalue: &'a str,
    pub timestamp: &'a str,
    pub value: &'a str,
}


impl<'a> std::fmt::Display for Event<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ \"type\": \"{}\", \"channel\": {}, \"id\": {}, \"value\": {}, \"raw\": \"{}\" }}",
            self.r#type, self.channel, self.id, self.value, bytes_to_strhex(self.raw, " "))
    }
}

impl std::fmt::Display for EventBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_event().fmt(f)
    }
}

impl EventBuf {
    pub fn as_event(&self) -> Event {
        Event {
            r#type: self.r#type,
            channel: self.channel,
            id: self.id,
            value: self.value,
            raw: &self.raw[..],
            timestamp: self.timestamp,
        }
    }
}

impl Into<EventBuf> for Event<'_> {
    fn into(self) -> EventBuf {
        EventBuf {
            r#type: self.r#type,
            channel: self.channel,
            id: self.id,
            value: self.value,
            raw: Vec::from(self.raw),
            timestamp: self.timestamp,
        }
    }
}

impl From<u8> for EventType {
    fn from(v: u8) -> Self {
        if ! (0b1000..=0b1111).contains(&v) {
            // not in defined space: unknown
            EventType::Unknown
        }
        else {
            // safe since all valid cases are defined
            unsafe { std::mem::transmute(v) }
        }
    }
}

fn bytes_to_strhex(bytes: &[u8], separator: &str) -> String {
    let mut s = String::new();
    for &byte in bytes {
        write!(&mut s, "{:02X}{}", byte, separator).expect("unexpected write error");
    }
    if s.ends_with(separator) {
        return s.trim_end_matches(separator).to_string();
    }
    s
}

impl<'a> Event<'a> {
    pub fn new() -> Self {
        Event {
            r#type: EventType::Unknown,
            channel: 0,
            id: 0,
            value: 0,
            raw: &[],
            timestamp: None,
        }
    }

    pub fn key(&self) -> u32 {
        event_to_key(self.r#type, self.channel, self.id)
    }

    pub fn make_env(&self, remap: Option<&Remapper<f64>>, float: bool) -> Result<EventEnv, Error>
    {
        Ok(EventEnv {
            channel: self.channel.to_string(),
            id: self.id.to_string(),
            rawvalue: self.value.to_string(),
            raw: bytes_to_strhex(self.raw, " "),
            timestamp: self.timestamp.unwrap_or(SystemTime::now()).duration_since(SystemTime::UNIX_EPOCH)?.as_secs_f64().to_string(),
            value: match (remap,float) {
                (Some(r),true)  => r.remap(self.value as f64).to_string(),
                (Some(r),false) => r.remap_to::<i64>(self.value as f64).unwrap().to_string(),
                _ => self.value.to_string(),
            }
        })
    }
}

impl<'a> From<&'a [u8]> for Event<'a> {
    fn from(v: &'a [u8]) -> Event<'a> {
        if v.is_empty() {
            eprintln!("warning: empty signal");
            return Default::default();
        }
        let event_type = EventType::from(v[0]/16);
        let channel = if event_type.has_channel() { v[0]%16 } else { 0 };
        let (id, value) = match event_type {
            EventType::PitchBend => {
                (0, (v[2] as u16)*256 + (v[1] as u16) )
            },
            EventType::Unknown => {
                eprintln!("warning: unknown signal type: {}", v[0]);
                (0,0)
            }
            EventType::System => (0,0),
            EventType::ChannelPressure => (0,v[1] as u16),
            EventType::ProgramChange => (v[1],0),
            EventType::NoteOn | EventType::NoteOff | EventType::PolyphonicKeyPressure | EventType::Controller => (v[1],(v[2] as u16)),
        };
        Event {
            r#type: event_type,
            channel,
            id,
            value,
            raw: v,
            timestamp: None,
        }
    }
}

impl EventEnv {
    pub fn to_map(self, m: Option<&EventEnvMap>) -> HashMap<&str,String> {
        let mut r = HashMap::new();
        let keys: EventEnvRef = match m {
            Some(v) => {
                EventEnvRef {
                    channel: v.channel.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.channel),
                    id: v.id.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.id),
                    raw: v.raw.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.raw),
                    rawvalue: v.rawvalue.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.rawvalue),
                    timestamp: v.timestamp.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.timestamp),
                    value: v.value.as_ref().map(|x| &x[..]).unwrap_or(EVENT_ENV_DEFAULT.value),
                }
            }
            _ => EVENT_ENV_DEFAULT.clone(),
        };
        r.insert(keys.channel, self.channel);
        r.insert(keys.id, self.id);
        r.insert(keys.raw, self.raw);
        r.insert(keys.rawvalue, self.rawvalue);
        r.insert(keys.timestamp, self.timestamp);
        r.insert(keys.value, self.value);
        r
    }
}
