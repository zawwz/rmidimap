use std::{collections::HashMap, time::SystemTime};
use std::fmt::Write;

use serde::{Serialize,Deserialize};

pub fn event_to_key(r#type: EventType, channel: u8, id: u8) -> u32 {
    (r#type as u32)*256*256 + (channel as u32)*256 + (id as u32)
}

#[repr(u8)]
#[derive(Serialize,Deserialize,Debug,Copy,Clone)]
pub enum EventType {
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
        match self {
            EventType::Unknown | EventType::ProgramChange | EventType::PitchBend | EventType::System => false,
            _ => true,
        }
    }
    pub fn has_channel(&self) -> bool {
        match self {
            EventType::Unknown | EventType::ProgramChange | EventType::System => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct Event<'a> {
    pub r#type: EventType,
    pub channel: u8,
    pub id: u8,
    pub value: u16,
    pub raw: &'a [u8],
    pub timestamp: Option<SystemTime>,
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

fn bytes_to_strhex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for &byte in bytes {
        write!(&mut s, "{:X} ", byte).expect("Unable to write");
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
    pub fn gen_env(&self) -> HashMap<&str, String> {
        let mut ret = HashMap::new();
        //TODO: type?
        ret.insert("channel", self.channel.to_string());
        ret.insert("id", self.id.to_string());
        ret.insert("value", self.value.to_string());
        ret.insert("raw", bytes_to_strhex(self.raw));
        ret.insert("timestamp", self.timestamp.unwrap_or(SystemTime::now()).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64().to_string());
        ret
    }
}

impl<'a> From<&'a [u8]> for Event<'a> {
    fn from(v: &'a [u8]) -> Event<'a> {
        let channel = v[0]%16;
        let event_type = EventType::from(v[0]/16);
        let (id, value) = match event_type {
            EventType::PitchBend => {
                (0, (v[2] as u16)*256 + (v[1] as u16) )
            },
            EventType::Unknown => {
                match v.len() > 0 {
                    true => eprintln!("warn: unknown signal type: {}", v[0]),
                    false => eprintln!("warn: empty signal"),
                };
                (0,0)
            }
            EventType::System => (0,0),
            EventType::PolyphonicKeyPressure |
            EventType::ChannelPressure |
            EventType::ProgramChange => {
                todo!()
            }
            EventType::NoteOn | EventType::NoteOff | EventType::Controller => (v[1],(v[2] as u16)),
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
