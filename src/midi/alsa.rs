extern crate libc;
extern crate alsa;

use std::{mem, thread};
use std::ffi::{CString, CStr};
use std::time::SystemTime;
use std::sync::mpsc;

use crate::midi::{MidiInput,MidiInputHandler,MidiPort,PortFilter,MidiPortHandler,MidiAddrHandler};

use alsa::{Seq, Direction};
use alsa::seq::{ClientIter, PortIter, MidiEvent, PortInfo, PortSubscribe, Addr, QueueTempo, EventType, PortCap, PortType};

use std::error::Error;

pub type DeviceAddr = alsa::seq::Addr;

pub fn get_ports(s: &Seq, capability: PortCap) -> Vec<PortInfo> {
    ClientIter::new(s).flat_map(|c| PortIter::new(s, c.get_client()))
        .filter(|p| p.get_capability().contains(capability))
        .collect()
}

mod helpers {
    pub fn poll(fds: &mut [super::libc::pollfd], timeout: i32) -> i32 {
        unsafe { super::libc::poll(fds.as_mut_ptr(), fds.len() as super::libc::nfds_t, timeout) }
    }
}

pub struct MidiInputAlsa {
    seq: Seq,
    queue_id: i32,
    subscription: Option<PortSubscribe>,
    connect_addr: Option<Addr>,
    stop_trigger: [i32;2],
}

impl Drop for MidiInputAlsa {
    fn drop(&mut self) {
        self.close_internal();
    }
}

impl MidiInputAlsa {
    fn init_trigger(&mut self) -> Result<(), Box<dyn Error>> {
        let mut trigger_fds = [-1, -1];
        if unsafe { self::libc::pipe(trigger_fds.as_mut_ptr()) } == -1 {
            todo!()
        } else {
            self.stop_trigger = trigger_fds;
            Ok(())
        }
    }


    fn init_queue(&mut self) -> i32 {
        let mut queue_id = 0;
        // Create the input queue
        if !cfg!(feature = "avoid_timestamping") {
            queue_id = self.seq.alloc_named_queue(unsafe { CStr::from_bytes_with_nul_unchecked(b"midir queue\0") }).unwrap();
            // Set arbitrary tempo (mm=100) and resolution (240)
            let qtempo = QueueTempo::empty().unwrap();
            qtempo.set_tempo(600_000);
            qtempo.set_ppq(240);
            self.seq.set_queue_tempo(queue_id, &qtempo).unwrap();
            let _ = self.seq.drain_output();
        }
        
        queue_id
    }

    fn start_input_queue(&mut self, queue_id: i32) {
        if !cfg!(feature = "avoid_timestamping") {
            let _ = self.seq.control_queue(queue_id, EventType::Start, 0, None);
            let _ = self.seq.drain_output();
        }
    }

    fn create_port(&mut self, port_name: &CStr, queue_id: i32) -> Result<i32, Box<dyn Error>> {
        let mut pinfo = PortInfo::empty().unwrap();
        // these functions are private, and the values are zeroed already by `empty()`
        //pinfo.set_client(0);
        //pinfo.set_port(0);
        pinfo.set_capability(PortCap::WRITE | PortCap::SUBS_WRITE);
        pinfo.set_type(PortType::MIDI_GENERIC | PortType::APPLICATION);
        pinfo.set_midi_channels(16);
        
        if !cfg!(feature = "avoid_timestamping") {
            pinfo.set_timestamping(true);
            pinfo.set_timestamp_real(true);
            pinfo.set_timestamp_queue(queue_id);
        }
        
        pinfo.set_name(port_name);
        match self.seq.create_port(&pinfo) {
            Ok(_) => Ok(pinfo.get_port()),
            Err(v) => Err(Box::new(v))
        }
    }

    fn close_internal(&mut self) 
    {
        if let Some(ref subscription) = self.subscription {
            let _ = self.seq.unsubscribe_port(subscription.get_sender(), subscription.get_dest());
        }
        
        // Stop and free the input queue
        if !cfg!(feature = "avoid_timestamping") {
            let _ = self.seq.control_queue(self.queue_id, EventType::Stop, 0, None);
            let _ = self.seq.drain_output();
            let _ = self.seq.free_queue(self.queue_id);
        }

        for fd in self.stop_trigger {
            if fd >= 0 {
                unsafe { self::libc::close(fd) };
            }
        }
    }

    fn signal_stop_input_internal(stop_trigger: i32) -> Result<(), Box<dyn Error>> {
        if unsafe { self::libc::write(stop_trigger, &false as *const bool as *const _, mem::size_of::<bool>() as self::libc::size_t) } == -1 {
            todo!()
        }
        Ok(())
    }

    fn alsa_input_handler<F, D>(&mut self, callback: F, mut userdata: D) -> Result<(), Box<dyn Error>> 
    where F: Fn(&Self, alsa::seq::Event, &mut D) -> bool {
        // fd defitions
        use self::alsa::PollDescriptors;
        use self::libc::pollfd;
        const INVALID_POLLFD: pollfd = pollfd {
            fd: -1,
            events: 0,
            revents: 0,
        };

        let mut seq_input = self.seq.input();

        // make poll fds
        let poll_desc_info = (&self.seq, Some(Direction::Capture));
        let mut poll_fds = vec![INVALID_POLLFD; poll_desc_info.count()+1];
        poll_fds[0] = pollfd {
            fd: self.stop_trigger[0],
            events: self::libc::POLLIN,
            revents: 0,
        };
        poll_desc_info.fill(&mut poll_fds[1..]).unwrap();

        loop {
            if let Ok(0) = seq_input.event_input_pending(true) {
                // No data pending: wait
                if helpers::poll(&mut poll_fds, -1) >= 0 {
                    // Read stop event from triggerer
                    if poll_fds[0].revents & self::libc::POLLIN != 0 {
                        let mut pollread = false; 
                        let _res = unsafe { self::libc::read(poll_fds[0].fd, mem::transmute(&mut pollread), mem::size_of::<bool>() as self::libc::size_t) };
                        if pollread == false {
                            break;
                        }
                    }
                }
                continue;
            }
            // get event
            let ev = seq_input.event_input()?;

            // handle disconnect event on watched port
            if ev.get_type() == EventType::PortUnsubscribed {
                if let Some(c) = ev.get_data::<alsa::seq::Connect>() {
                    if c.sender == self.connect_addr.unwrap() {
                        break;
                    }
                }
            }

            if (callback)(self, ev, &mut userdata) {
                break;
            }
        }
        Ok(())
    }

    fn handle_input_internal<F, D>(&mut self, callback: F, userdata: D) -> Result<(), Box<dyn Error>> 
    where F: Fn(Option<SystemTime>, &[u8], &mut D) + Send {
        let decoder = MidiEvent::new(0).unwrap();
        decoder.enable_running_status(false);

        let message = vec!();
        let buffer: [u8;12] = [0;12];
        let continue_sysex = false;

        self.alsa_input_handler(|_, mut ev, (message, buffer, continue_sysex, userdata)| {
            if !*continue_sysex { message.clear() }

            let do_decode = match ev.get_type() {
                EventType::PortSubscribed |
                EventType::PortUnsubscribed | 
                EventType::Qframe |
                EventType::Tick |
                EventType::Clock |
                EventType::Sensing => false,
                EventType::Sysex => {
                    message.extend_from_slice(ev.get_ext().unwrap());
                    *continue_sysex = *message.last().unwrap() != 0xF7;
                    false
                }
                _ => true
            };

            // NOTE: SysEx messages have already been "decoded" at this point!
            if do_decode {
                let nbytes = decoder.decode(buffer, &mut ev).unwrap();
                if nbytes > 0 {
                    message.extend_from_slice(&buffer[0..nbytes+1]);
                }
            }

            if message.len() == 0 || *continue_sysex { return false; }

            let alsa_time = ev.get_time().unwrap();
            let secs = alsa_time.as_secs();
            let nsecs = alsa_time.subsec_nanos();
            let timestamp = ( secs as u64 * 1_000_000 ) + ( nsecs as u64 / 1_000 );
            //TODO: translate to SystemTime?
            (callback)(None, &message, userdata);
            false
        }
        , (message, buffer, continue_sysex, userdata))?;
        Ok(())
    }
}

impl MidiInput<Addr> for MidiInputAlsa {
    fn new(client_name: &str) -> Result<Self, Box<dyn Error>> {
        let seq = match Seq::open(None, None, true) {
            Ok(s) => s,
            Err(_) => todo!(),
        };
        
        let c_client_name = CString::new(client_name)?;
        seq.set_client_name(&c_client_name)?;
        
        Ok(MidiInputAlsa {
            seq: seq,
            queue_id: 0,
            subscription: None,
            connect_addr: None,
            stop_trigger: [-1,-1],
        })
    }

    fn close(mut self) -> Result<(), Box<dyn Error>> {
        self.close_internal();
        Ok(())
    }


    fn ports_handle(&self) -> Vec<MidiPortHandler> {
        get_ports(&self.seq, PortCap::READ | PortCap::SUBS_READ).iter().map(|x| {
            let cinfo = self.seq.get_any_client_info(x.get_client()).unwrap();
            MidiPortHandler::ALSA( MidiPort{
                name: cinfo.get_name().unwrap().to_string()+":"+x.get_name().unwrap(),
                addr: x.addr(),
            })
        }).collect()
    }

    fn ports(&self) -> Vec<MidiPort<Addr>> {
        get_ports(&self.seq, PortCap::READ | PortCap::SUBS_READ).iter().map(|x| {
            let cinfo = self.seq.get_any_client_info(x.get_client()).unwrap();
            MidiPort {
                name: cinfo.get_name().unwrap().to_string()+":"+x.get_name().unwrap(),
                addr: x.addr(),
            }
        }).collect()
    }

    fn filter_ports(&self, mut ports: Vec<MidiPort<Addr>>, filter: PortFilter) -> Vec<MidiPort<Addr>> {
        ports.retain(
            |p| {
                match &filter {
                    PortFilter::Name(s) => p.name.find(s).is_some(),
                    PortFilter::Addr(MidiAddrHandler::ALSA(s)) => p.addr == *s,
                    _ => todo!(),
                }
            }
        );
        ports
    }

    fn connect(&mut self, port_addr: &Addr, port_name: &str) -> Result<(), Box<dyn Error>> {
        let src_pinfo = self.seq.get_any_port_info(*port_addr)?;
        let queue_id = self.init_queue();
        let c_port_name = CString::new(port_name)?;
        let vport = self.create_port(&c_port_name, queue_id)?;

        let sub = PortSubscribe::empty().unwrap();
        sub.set_sender(src_pinfo.addr());
        sub.set_dest(Addr { client: self.seq.client_id().unwrap(), port: vport});
        self.seq.subscribe_port(&sub)?;
        self.subscription = Some(sub);
        self.init_trigger()?;
        self.connect_addr = Some(*port_addr);
        self.start_input_queue(queue_id);
        Ok(())
    }

    fn device_events(&mut self, ts: mpsc::Sender<MidiPortHandler>) -> Result<(), Box<dyn Error>> {
        let ports = self.ports();
        let port = self.filter_ports(ports, PortFilter::Name("System:Announce".to_string()));
        self.connect(&port[0].addr, "rmidimap-alsa-announce")?;
        self.alsa_input_handler(|s, ev, _|{
            // handle disconnect event on watched port
            match ev.get_type() {
                // EventType::PortStart | EventType::ClientStart | EventType::PortExit | EventType::ClientExit => {
                EventType::PortStart => {
                    if let Some(a) = ev.get_data::<alsa::seq::Addr>() {
                        let p = s.ports();
                        let pp = s.filter_ports(p, PortFilter::Addr( MidiAddrHandler::ALSA(a.clone()) ));
                        if pp.len() > 0 {
                            ts.send(MidiPortHandler::ALSA(pp[0].clone())).unwrap();
                        }
                    };
                    false
                }
                _ => false,
            }
        }, ())?;
        self.close_internal();
        Ok(())
    }
}

impl MidiInputHandler for MidiInputAlsa
{
    fn signal_stop_input(&self) -> Result<(), Box<dyn Error>> {
        if unsafe { self::libc::write(self.stop_trigger[1], &false as *const bool as *const _, mem::size_of::<bool>() as self::libc::size_t) } == -1 {
            todo!()
        }
        Ok(())
    }

    fn handle_input<F, D>(&mut self, callback: F, (rs, ts): (mpsc::Receiver<bool>, mpsc::Sender<bool>), userdata: D) -> Result<(), Box<dyn Error>> 
    where 
        F: Fn(Option<SystemTime>, &[u8], &mut D) + Send,
        D: Send,
    {
        thread::scope( |sc| -> Result<(), Box<dyn Error>> {
            let stop_trigger = self.stop_trigger[1];
            let t = sc.spawn(move || {
                let userdata = userdata;
                self.handle_input_internal(callback, userdata).unwrap();
                ts.send(false).unwrap();
            });
            match rs.recv()? {
                true => Self::signal_stop_input_internal(stop_trigger)?,
                false => ()
            };
            t.join().unwrap();
            Ok(())
        })
    }
}

