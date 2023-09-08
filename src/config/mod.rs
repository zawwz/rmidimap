pub mod event;
pub mod device;
pub mod run;
pub mod serializer;

use serializer::ConfigSerializer;

use std::str::FromStr;

use crate::util;

pub use device::DeviceConfig;
pub use event::EventConfig;
pub use run::RunConfig;
pub type EventEnvMap = serializer::EventEnvSerializer;

#[derive(Clone,Debug)]
pub struct Config {
    pub log: bool,
    pub driver: Option<crate::midi::MidiDriver>,
    pub devices: Vec<DeviceConfig>,
}

impl TryFrom<ConfigSerializer> for Config {
    type Error = crate::Error;
    fn try_from(v: ConfigSerializer) -> Result<Self, Self::Error> {
        Ok(Config {
            log: v.log_devices.unwrap_or(false),
            driver: v.driver,
            devices: util::map_tryfrom(v.devices)?,
        })
    }
}

impl TryFrom<&[u8]> for Config {
    type Error = crate::Error;
    fn try_from(dat: &[u8]) -> Result<Self, Self::Error> {
        let c: ConfigSerializer = serde_yaml::from_slice(dat)?;
        Ok(Config::try_from(c)?)
    }
}

impl FromStr for Config {
    type Err = crate::Error;
    fn from_str(dat: &str) -> Result<Self, Self::Err> {
        let c: ConfigSerializer = serde_yaml::from_str(dat)?;
        Ok(Config::try_from(c)?)
    }
}
