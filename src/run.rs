use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex,Arc};

use crate::Error;

use crate::midi::{PortFilter,MidiHandler,MidiPortHandler};
use crate::config::{Config,DeviceConfig};
use crate::eventmap::EventMap;

type DeviceRunItem<'a> = (&'a DeviceConfig, EventMap<'a>, Option<Arc<Mutex<(u32, u32)>>>);
type DeviceRunResult<'a> =(thread::ScopedJoinHandle<'a, ()>, mpsc::Sender<bool>);

pub fn cross_shell(cmd: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!("cmd", "/C", cmd)
    } else {
        vec!("sh", "-c", cmd)
    }
    .iter().map(
        |x| x.to_string()
    ).collect()
}

pub fn run_config(conf: &Config) -> Result<(), Error> {
    let cfevmap: Vec<DeviceRunItem> = conf.devices.iter().map(|x|
        (x, EventMap::from(x),
            x.max_connections.map(|v| (Arc::new(Mutex::new((0,v)))))
        )
    ).collect();

    let input = MidiHandler::new("rmidimap")?;

    thread::scope(|s| -> Result<(), Error> {
        let (tdev,rdev) = mpsc::channel::<MidiPortHandler>();
        let mut threads: Vec<(thread::ScopedJoinHandle<'_, ()>, mpsc::Sender<bool>)> = Vec::new();
        let ports = input.ports()?;
        for p in ports {
            if let Some(v) = try_connect_process(&input, s, &p, &cfevmap)? { threads.push(v) }
        }
        let _event_thread = s.spawn(|| {
            let mut input = MidiHandler::new("rmidimap-event-watcher").unwrap();
            input.device_events(tdev).unwrap();
        });
        loop {
            let p = rdev.recv()?;
            if let Some(v) = try_connect_process(&input, s, &p, &cfevmap)? { threads.push(v) }
        }
    })?;
    Ok(())
}

fn try_connect_process<'a>(
    input: &MidiHandler,
    s: &'a thread::Scope<'a, '_>,
    p: &MidiPortHandler,
    cfevmap: &'a[DeviceRunItem<'a>],
    )
        -> Result<Option<DeviceRunResult<'a>>, Error> {
    for (dev, eventmap, m) in cfevmap {
        if let Some(m) = m {
            let m = m.lock().unwrap();
            if m.0 >= m.1 {
                continue;
            }
        }
        if let Some(mut c) = input.try_connect(p.clone(), PortFilter::from(*dev))? {
            if let Some(m) = m {
                let mut m = m.lock().unwrap();
                m.0 += 1;
            }
            let (sts,srs) = mpsc::channel::<bool>();
            let nsts = sts.clone();
            let mm = m.as_ref().map(Arc::clone);
            let t = s.spawn( move || {
                dev.run_connect().unwrap();
                c.run(eventmap, (srs,nsts)).unwrap();
                if let Some(m) = mm {
                    let mut m = m.lock().unwrap();
                    m.0 -= 1;
                }
                dev.run_disconnect().unwrap();
            });
            return Ok(Some((t, sts)));
        }
    }
    Ok(None)
}
