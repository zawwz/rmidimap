pub mod alsa;

use crate::midi::alsa::MidiInputAlsa;

extern crate libc;

use crate::config::DeviceConfig;
use crate::eventmap::EventMap;
use crate::event::Event;

use std::error::Error;
use std::time::SystemTime;
use std::sync::mpsc;

#[derive(Eq,PartialEq,Debug,Clone)]
pub struct MidiPort<T>{
    pub name: String,
    pub addr: T,
}

#[derive(Debug,Clone)]
pub enum PortFilter {
    Name(String),
    Regex(String),
    Addr(MidiAddrHandler),
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub enum MidiHandlerDriver {
    ALSA,
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub enum MidiAddrHandler {
    ALSA(alsa::DeviceAddr),
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub enum MidiPortHandler {
    ALSA(MidiPort<alsa::DeviceAddr>),
}

pub enum MidiHandler {
    ALSA(MidiInputAlsa),
}

impl From<MidiPortHandler> for MidiAddrHandler {
    fn from(a: MidiPortHandler) -> Self {
        match a {
            MidiPortHandler::ALSA(p) => MidiAddrHandler::ALSA(p.addr),
        }
    }
}

impl From<&DeviceConfig> for PortFilter {
    fn from(conf: &DeviceConfig) -> Self {
        if conf.name.is_some() {
            PortFilter::Name(conf.name.clone().unwrap_or(String::new()))
        }
        else {
            todo!()
        }
    }
}

mod helper {
}


pub trait MidiInput<T> {
    fn new(client_name: &str) -> Result<Self, Box<dyn Error>> 
    where Self: Sized;

    fn close(self) -> Result<(), Box<dyn Error>>;

    fn ports(&self) -> Vec<MidiPort<T>>;
    fn ports_handle(&self) -> Vec<MidiPortHandler>;

    fn filter_ports<'a>(&self, ports: Vec<MidiPort<T>>, filter: PortFilter) -> Vec<MidiPort<T>>;

    fn connect(&mut self, port_addr: &T, port_name: &str) -> Result<(), Box<dyn Error>>;

    fn device_events(&mut self, ts: mpsc::Sender<MidiPortHandler>) -> Result<(), Box<dyn Error>>;
}

pub trait MidiInputHandler {
    fn signal_stop_input(&self) -> Result<(), Box<dyn Error>>;

    fn handle_input<F, D>(&mut self, callback: F, rts: (mpsc::Receiver<bool>, mpsc::Sender<bool>), userdata: D) -> Result<(), Box<dyn Error>> 
    where 
        F: Fn(Option<SystemTime>, &[u8], &mut D) + Send,
        D: Send,
    ;  
}

macro_rules! handler_try_connect {
    ( $m:expr , $filter:expr, $port:expr,  $( $handler:ident ),+ ) => {
        match $m {
            $(
            MidiHandler::$handler(v) => {
                match $port {
                    MidiPortHandler::$handler(_) => {
                        let maddr = MidiAddrHandler::from($port);
                        let portmap = v.ports();
                        let pv = v.filter_ports(portmap, PortFilter::Addr(maddr));
                        let pv = v.filter_ports(pv, $filter);
                        if pv.len() > 0 {
                            let port = &pv[0];
                            let mut h = MidiHandler::new_with_driver("rmidimap-handler", MidiHandlerDriver::$handler)?;
                            println!("Connect to device {}", port.name);
                            match &mut h {
                                MidiHandler::$handler(v) => { 
                                    v.connect(&port.addr, "rmidimap-handler").unwrap();
                                    Ok(Some(h))
                                }
                                _ => panic!("unexpected midi driver failure"),
                            }
                        }
                        else {
                            Ok(None)
                        }
                    },
                    _ => panic!("unexpected midi driver failure"),
                }
            }
            )*
        }
    };
}

macro_rules! handler_fcall {
    ( $m:expr , $fct:expr , $arg:expr ,  $( $handler:ident ),+ ) => {
        match $m {
            $(
            MidiHandler::$handler(v) => $fct(v, $arg),
            )*
        }
    };
}

impl MidiHandler {
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        Self::new_with_driver(name, MidiHandlerDriver::ALSA)
    }

    pub fn new_with_driver(name: &str, driver: MidiHandlerDriver) -> Result<Self, Box<dyn Error>> {
        match driver {
            MidiHandlerDriver::ALSA => Ok(MidiHandler::ALSA(MidiInputAlsa::new(name)?)),
        }
    }

    pub fn ports(&self) -> Result<Vec<MidiPortHandler>, Box<dyn Error>> {
        handler_fcall!{
            self, handle_port_list ,(),
            ALSA
        }
    }

    pub fn try_connect(&self, addr: MidiPortHandler, filter: PortFilter) -> Result<Option<MidiHandler>, Box<dyn Error>> {
        let r: Result<Option<MidiHandler>, Box<dyn Error>> = handler_try_connect!{
            self, filter, addr,
            ALSA
        };
        r
    }

    pub fn run(&mut self, eventmap: &EventMap, (rs,ts): (mpsc::Receiver<bool>, mpsc::Sender<bool>)) -> Result<(), Box<dyn Error>>  {
        handler_fcall!{
            self, handle_inputport ,(eventmap,(rs,ts)),
            ALSA
        }
    }

    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        handler_fcall!{
            self, handle_signal_stop, (),
            ALSA
        }  
    }

    pub fn device_events(&mut self, ts: mpsc::Sender<MidiPortHandler>) -> Result<(), Box<dyn Error>> {
        handler_fcall!{
            self, device_events, ts,
            ALSA
        }  
    }
}

fn handle_port_list<T, A>(input: &T, _: ()) -> Result<Vec<MidiPortHandler>, Box<dyn Error>> 
where T: MidiInput<A>
{
    Ok(input.ports_handle())
}

fn handle_inputport<T>(input: &mut T, (eventmap, (rs, ts)): (&EventMap, (mpsc::Receiver<bool>, mpsc::Sender<bool>))) -> Result<(), Box<dyn Error>> 
where T: MidiInputHandler
{
    input.handle_input(|t,m,_| {
        let mut event = Event::from(m);
        event.timestamp = t;
        eventmap.run_event(&event).unwrap();
    }, (rs,ts), ())?;
    Ok(())
}

fn handle_signal_stop<T>(input: &T, _: ()) -> Result<(), Box<dyn Error>> 
where T: MidiInputHandler
{
    input.signal_stop_input()
}

fn device_events<T, A>(input: &mut T, ts: mpsc::Sender<MidiPortHandler>) -> Result<(), Box<dyn Error>> 
where T: MidiInput<A>
{
    input.device_events(ts)
}

