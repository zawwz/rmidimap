pub mod event;
pub mod device;
pub mod run;
pub mod eventenv;

pub type DeviceConfigSerializer = device::DeviceConfigSerializer;
pub type EventConfigSerializer = event::EventConfigSerializer;
pub type RunConfigSerializer = run::RunConfigSerializer;
pub type EventEnvSerializer = eventenv::EventEnvSerializer;

use serde::Deserialize;

#[derive(Deserialize,Clone,Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigSerializer {
    pub devices: Vec<DeviceConfigSerializer>,
}
