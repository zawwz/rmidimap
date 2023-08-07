use crate::config::{RunConfig,EventConfig};
use crate::event::Event;

use serde::Deserialize;

#[derive(Deserialize,Debug,Clone)]
pub struct DeviceConfig {
    pub name: Option<String>,
    pub regex: Option<String>,
    pub connect: Option<Vec<RunConfig>>,
    pub disconnect: Option<Vec<RunConfig>>,
    pub events: Option<Vec<EventConfig>>,
    pub multiconnect: Option<bool>,
}

//impl DeviceConfig {
//    fn connect(&self, port: &MidiInputPort) {
//        let mut midi_in = MidiInput::new("midi inputs")?;
//        midi_in.ignore(Ignore::None);
//        let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, emap| {
//            let event = event::Event::from(message);
//            emap.run_event(&event).unwrap();
//        }, eventmap)?;
//    }
//}

impl DeviceConfig {
    fn run_internal<'a, T>(&self, v: Option<T>) -> Result<Vec<std::process::ExitStatus>, std::io::Error>
    where
        T: IntoIterator<Item = &'a RunConfig>
    {
        let mut r = Vec::new();
        if let Some(ev) = v {
            for e in ev {
                r.push( e.run(Event::new().gen_env())? ) ;
            }
        }
        Ok(r)
    }

    pub fn run_connect(&self) -> Result<Vec<std::process::ExitStatus>, std::io::Error> {
        self.run_internal(self.connect.as_ref())
    }

    pub fn run_disconnect(&self) -> Result<Vec<std::process::ExitStatus>, std::io::Error>  {
        self.run_internal(self.disconnect.as_ref())
    }
}