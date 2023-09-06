use std::sync::mpsc;
use std::thread;
use std::sync::{Mutex,Arc};

use libc::SIGUSR1;
use signal_hook::iterator::Signals;

use crate::Error;
use crate::midi::{PortFilter,MidiHandler,MidiPortHandler};
use crate::config::{Config,DeviceConfig};
use crate::eventmap::EventMap;

type DeviceRunItem<'a> = (&'a DeviceConfig, EventMap<'a>, Option<Arc<Mutex<(u32, u32)>>>);
type DeviceRunResult<'a> =(thread::ScopedJoinHandle<'a, Result<(), Error>>, mpsc::Sender<bool>);

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

pub fn list_devices() -> Result<(), Error> {
    let input = MidiHandler::new("rmidimap")?;
    let ports = input.ports()?;
    for p in ports {
        println!("{}", p);
    }
    Ok(())
}

pub fn run_config(conf: &Config) -> Result<(), Error> {
    let cfevmap: Vec<DeviceRunItem> = conf.devices.iter().map(|x|
        (x, EventMap::from(x),
            x.max_connections.map(|v| (Arc::new(Mutex::new((0,v)))))
        )
    ).collect();

    let input = MidiHandler::new("rmidimap")?;

    let (tdev,rdev) = mpsc::channel::<Option<MidiPortHandler>>();
    let (tsd,rsd) = mpsc::channel::<bool>();

    let ntsd = tsd.clone();
    let ntdev = tdev.clone();
    let mut signals = Signals::new(&[SIGUSR1])?;
    let _signal_thread = thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                10 => {
                    println!("Recieved SIGUSR1, reloading config file");
                    ntsd.send(true).unwrap();
                    ntdev.send(None).unwrap();
                    break;
                }
                _ => (),
            }
        }
    });

    thread::scope(|s| -> Result<(), Error> {
        let mut threads: Vec<DeviceRunResult> = Vec::new();
        let ports = input.ports()?;
        for p in ports {
            if let Some(v) = try_connect_process(&input, s, &p, &cfevmap)? { threads.push(v) }
        }

        let event_thread = s.spawn(move || {
            let mut input = MidiHandler::new("rmidimap-event-watcher").unwrap();
            let r = input.device_events(tdev.clone(), (tsd,rsd));
            tdev.send(None).unwrap();
            r
        });

        loop {
            let p = rdev.recv()?;
            if p.is_none() {
                break;
            }
            if let Some(v) = try_connect_process(&input, s, &p.unwrap(), &cfevmap)? { threads.push(v) }
        };
        event_thread.join().unwrap()?;
        for (thread,ss) in threads {
            let _ = ss.send(true);
            let _ = thread.join().unwrap().map_err(|e| eprintln!("WARN: error in thread: {}", e));
        }
        Ok(())
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
    for (dev, eventmap, counter) in cfevmap {
        // device counter is full
        if let Some(m) = counter {
            let m = m.lock().unwrap();
            if m.0 >= m.1 {
                continue;
            }
        }

        if let Some(mut c) = input.try_connect(p.clone(), PortFilter::from(*dev))? {
            // increase device counter
            if let Some(m) = counter {
                let mut m = m.lock().unwrap();
                m.0 += 1;
            }
            // stop signal channel
            let (sts,srs) = mpsc::channel::<bool>();
            let mm = counter.as_ref().map(Arc::clone);
            let nsts = sts.clone();
            let t = s.spawn( move || -> Result<(), Error> {
                dev.run_connect()?;
                // blocking process
                c.run(dev, eventmap, (nsts,srs))?;
                // decrease device counter
                if let Some(m) = mm {
                    let mut m = m.lock().unwrap();
                    m.0 -= 1;
                }
                dev.run_disconnect()?;
                Ok(())
            });
            return Ok(Some((t, sts)));
        }
    }
    Ok(None)
}
