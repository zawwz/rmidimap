use std::time::Duration;

use crate::event::Event;
use crate::util;
use crate::Error;
use super::{RunConfig,EventConfig};
use super::serializer::DeviceConfigSerializer;

#[derive(Debug,Clone)]
pub enum Identifier {
    All,
    Name(String),
    Regex(regex::Regex),
    Addr(String),
}

#[derive(Debug,Clone)]
pub struct DeviceConfig {
    pub identifier: Identifier,
    pub max_connections: Option<u32>,
    pub connect: Option<Vec<RunConfig>>,
    pub disconnect: Option<Vec<RunConfig>>,
    pub events: Option<Vec<EventConfig>>,
    pub queue_length: usize,
    pub interval: Duration,
}

impl DeviceConfig {
    fn run_internal<'a, T>(&self, v: Option<T>) -> Result<Vec<std::process::ExitStatus>, Error>
    where
        T: IntoIterator<Item = &'a RunConfig>
    {
        let mut r = Vec::new();
        if let Some(ev) = v {
            for e in ev {
                if let Some(v) = e.run(Event::new().make_env(None, false)?.to_map(e.envconf.as_ref()))? {
                    r.push(v);
                }
            }
        }
        Ok(r)
    }

    pub fn run_connect(&self) -> Result<Vec<std::process::ExitStatus>, Error> {
        self.run_internal(self.connect.as_ref())
    }

    pub fn run_disconnect(&self) -> Result<Vec<std::process::ExitStatus>, Error>  {
        self.run_internal(self.disconnect.as_ref())
    }
}

impl TryFrom<DeviceConfigSerializer> for DeviceConfig {
    type Error = crate::Error;
    fn try_from(v: DeviceConfigSerializer) -> Result<Self, Self::Error> {
        Ok(DeviceConfig {
            identifier: {
                match (v.name, v.regex, v.addr) {
                    (Some(_), Some(_), _      ) => return Err(Error::IncompatibleArgs("name","regex")),
                    (Some(_), None   , Some(_)) => return Err(Error::IncompatibleArgs("name","addr")),
                    (None   , Some(_), Some(_)) => return Err(Error::IncompatibleArgs("regex","addr")),
                    (Some(n), None,    None   ) => Identifier::Name(n),
                    (None,    Some(r), None   ) => Identifier::Regex(regex::Regex::new(&r)?),
                    (None,    None   , Some(a)) => Identifier::Addr(a),
                    (None,    None,    None   ) => Identifier::All,
                }
            },
            max_connections: v.max_connections,
            connect:    util::map_opt_tryfrom(v.connect)?,
            disconnect: util::map_opt_tryfrom(v.disconnect)?,
            events:     util::map_opt_tryfrom(v.events)?,
            queue_length: v.queue_length.unwrap_or(256),
            interval: v.interval.map(|x| x.unwrap()).unwrap_or_else(|| Duration::new(0, 0)),
        })
    }
}
