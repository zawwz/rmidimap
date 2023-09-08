use crate::util::InternalTryFrom;
use crate::{Error, constant};
use crate::config::DeviceConfig;
use crate::eventmap::EventMap;
use crate::event::{Event, EventBuf};

use std::str::FromStr;
use std::thread;
use std::time::{SystemTime, Instant};
use std::sync::{mpsc, Mutex, Arc};

use queues::{CircularBuffer, IsQueue};

use super::{PortFilter, MidiPort};

pub trait MidiInput
where
    <Self as MidiInput>::DeviceAddr: Clone+Send+FromStr,
    <Self as MidiInput>::DeviceAddr: InternalTryFrom<std::string::String>
{
    type DeviceAddr;
    fn new(client_name: &str) -> Result<Self, Error>
    where Self: Sized;

    fn close(self) -> Result<(), Error>;

    fn ports(&self) -> Result<Vec<MidiPort<Self::DeviceAddr>>, Error>;

    fn filter_ports(&self, ports: Vec<MidiPort<Self::DeviceAddr>>, filter: PortFilter<Self::DeviceAddr>) -> Vec<MidiPort<Self::DeviceAddr>>;

    fn connect(&mut self, port_addr: &Self::DeviceAddr, port_name: &str) -> Result<(), Error>;

    fn device_events(&mut self, ts: mpsc::Sender<Option<MidiPort<Self::DeviceAddr>>>, ss: (mpsc::Sender<bool>, mpsc::Receiver<bool>)) -> Result<(), Error>;

    fn signal_stop_input(&self) -> Result<(), Error>;

    fn handle_input<F, D>(&mut self, callback: F, rts: (mpsc::Sender<bool>, mpsc::Receiver<bool>), userdata: D) -> Result<(), Error>
    where
        F: Fn(&Self, &[u8], Option<SystemTime>, &mut D) + Send + Sync,
        D: Send,
    ;
}

pub trait MidiInputHandler 
where
    Self: Sized,
    <Self as MidiInputHandler>::DeviceAddr: Clone+Send+FromStr,
{
    type DeviceAddr;
    fn new(client_name: &str) -> Result<Self, Error>;
    fn ports(&self) -> Result<Vec<MidiPort<Self::DeviceAddr>>, Error>;
    fn try_connect(&self, port: MidiPort<Self::DeviceAddr>, filter: PortFilter<Self::DeviceAddr> ) -> Result<Option<Self>, Error>;
    fn run(&mut self, conf: &DeviceConfig, eventmap: &EventMap, trs: (mpsc::Sender<bool>, mpsc::Receiver<bool>)) -> Result<(), Error>;
    fn device_events(&mut self, ts: mpsc::Sender<Option<MidiPort<Self::DeviceAddr>>>, ss: (mpsc::Sender<bool>,mpsc::Receiver<bool>)) -> Result<(), Error>;
}

// Generic implementation

impl<T> MidiInputHandler for T
where
    T: MidiInput + Send,
    <T as MidiInput>::DeviceAddr: FromStr,
{
    type DeviceAddr = T::DeviceAddr;
    

    fn new(client_name: &str) -> Result<Self, Error> {
        MidiInput::new(client_name)
    }

    fn ports(&self) -> Result<Vec<MidiPort<Self::DeviceAddr>>, Error> {
        MidiInput::ports(self)
    }

    fn try_connect(&self, port: MidiPort<Self::DeviceAddr>, filter: PortFilter<T::DeviceAddr> ) -> Result<Option<Self>, Error> {
        let portmap = self.ports()?;
        let pv = self.filter_ports(portmap, PortFilter::Addr(port.addr));
        let pv = self.filter_ports(pv, filter);
        if pv.len() > 0 {
            let port = &pv[0];
            let mut v = T::new(constant::CLIENT_NAME_HANDLER)?;
            v.connect(&port.addr, constant::CLIENT_NAME_HANDLER)?;
            Ok(Some(v))
        }
        else {
            Ok(None)
        }
    }

    fn device_events(&mut self, ts: mpsc::Sender<Option<MidiPort<Self::DeviceAddr>>>, ss: (mpsc::Sender<bool>,mpsc::Receiver<bool>)) -> Result<(), Error> {
        self.device_events(ts, ss)
    }

    fn run(&mut self, conf: &DeviceConfig, eventmap: &EventMap, (ts, rs): (mpsc::Sender<bool>, mpsc::Receiver<bool>)) -> Result<(), Error> {
        thread::scope(|s| -> Result<(), Error> {

            // parking signal for runner, true = stop
            let (pts,prs) = mpsc::channel::<bool>();
    
            // event queue populated by the main thread and consumed by the exec thread
            let evq = Arc::new(Mutex::new(CircularBuffer::<EventBuf>::new(conf.queue_length)));
    
            // background execution loop
            let rq = evq.clone();
            let exec_thread = s.spawn(move || -> Result<(),Error> {
                loop {
                    if prs.recv()? {
                        break;
                    }
                    loop {
                        // nest the lock into a scope to release it before run
                        let (ev,start): (EventBuf,Instant) = {
                            let mut evq = rq.lock().unwrap();
                            if evq.size() > 0 {
                                (evq.remove().unwrap(), Instant::now())
                            } else {
                                break;
                            }
                        };
                        eventmap.run_event(&ev.as_event()).unwrap_or_else(|e| eprintln!("ERROR: error on run: {}", e) );
                        // wait until interval has been reached
                        let elapsed_time = start.elapsed();
                        if elapsed_time < conf.interval {
                            thread::sleep(conf.interval - elapsed_time);
                        }
                    }
                }
                Ok(())
            });
    
            self.handle_input(|_,m,t,(evq,pts)| {
                let mut event: EventBuf = Event::from(m).into();
                event.timestamp = t;
                if conf.log {
                    println!("{}: event: {}", constant::CLIENT_NAME, event);
                }
                let mut evq = evq.lock().unwrap();
                evq.add(event).unwrap();
                pts.send(false).expect("unexpected write error");
            }, (ts,rs), (evq,pts.clone()))?;
    
            pts.send(true).expect("unexpected write error");
            let _ = exec_thread.join();
    
            Ok(())
    
        })?;
        Ok(())
    }
}
