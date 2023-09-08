pub mod event;
pub mod device;
pub mod run;
pub mod eventenv;

pub use device::DeviceConfigSerializer;
pub use event::EventConfigSerializer;
pub use run::RunConfigSerializer;
pub use eventenv::EventEnvSerializer;

use serde::Deserialize;

#[derive(Deserialize,Clone,Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigSerializer {
    pub log_devices: Option<bool>,
    pub driver: Option<crate::midi::MidiDriver>,
    pub devices: Vec<DeviceConfigSerializer>,
}
