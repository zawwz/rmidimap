use std::sync::mpsc;
use std::error::Error;
use std::path::Path;
use std::thread;
use std::sync::Mutex;
use std::rc::Rc;

pub mod config;
pub mod run;
pub mod event;
pub mod eventmap;
pub mod midi;
pub mod util;
pub mod cli;

use clap::Parser;

use midi::{MidiHandler,MidiPortHandler};
use config::{Config,DeviceConfig};
use eventmap::EventMap;
use cli::Cli;

fn main() {
    let c = Cli::parse();
    match run(&c.map_file) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run(filepath: &Path) -> Result<(), Box<dyn Error>> {
    println!("Load file {}", filepath.to_str().unwrap());
    let dat = std::fs::read( filepath )?;
    
    let conf: Config = serde_yaml::from_slice(&dat)?;
    let cfevmap: Vec<(&DeviceConfig, EventMap, Rc<Mutex<bool>>)> = conf.devices.iter().map(|x| 
        (x, eventmap::EventMap::from(x), Rc::new(Mutex::new(false)))
    ).collect();

    let input = MidiHandler::new("rmidimap")?;
    
    thread::scope(|s| -> Result<(), Box<dyn Error>> {
        let (tdev,rdev) = mpsc::channel::<MidiPortHandler>();
        let mut threads: Vec<(thread::ScopedJoinHandle<'_, ()>, mpsc::Sender<bool>)> = Vec::new();
        let ports = input.ports()?;
        // TODO: centralize connection process in one place
        // TODO: "multiconnect=false" handling
        for p in ports {
            for (dev, eventmap, m) in &cfevmap {
                if let Some(mut c) = input.try_connect(p.clone(), midi::PortFilter::from(*dev))? {
                    let (sts,srs) = mpsc::channel::<bool>();
                    let nsts = sts.clone();
                    let t = s.spawn( move || {
                        dev.run_connect().unwrap();
                        c.run(eventmap, (srs,nsts)).unwrap();
                        dev.run_disconnect().unwrap();
                    });
                    threads.push((t, sts));
                    break;
                }
            }
        }
        let _event_thread = s.spawn(|| {
            let mut input = MidiHandler::new("rmidimap-event-watcher").unwrap();
            input.device_events(tdev).unwrap();
        });
        loop {
            let e = rdev.recv()?;
            for (dev, eventmap, m) in &cfevmap {
                if let Some(mut c) = input.try_connect(e.clone(), midi::PortFilter::from(*dev))? {
                    let (sts,srs) = mpsc::channel::<bool>();
                    let nsts = sts.clone();
                    let t = s.spawn( move || {
                        dev.run_connect().unwrap();
                        c.run(eventmap, (srs,nsts)).unwrap();
                        dev.run_disconnect().unwrap();
                    });
                    threads.push((t, sts));
                    break;
                }
            }
        }
    })?;
    Ok(())
}
