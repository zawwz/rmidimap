pub mod event;
pub mod device;
pub mod run;

pub type DeviceConfig = device::DeviceConfig;
pub type EventConfig = event::EventConfig;
pub type RunConfig = run::RunConfig;

use serde::Deserialize;

#[derive(Deserialize,Clone,Debug)]
pub struct Config {
    pub devices: Vec<DeviceConfig>,
}
