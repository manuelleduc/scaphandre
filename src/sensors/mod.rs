pub mod powercap_rapl;
pub mod units;

use std::error::Error;

use std::collections::HashMap;
use std::{fmt, fs};
use std::time::SystemTime;


pub trait RecordGenerator{
    fn get_record(&self) -> Record;
}

// !!!!!!!!!!!!!!!!! Topology !!!!!!!!!!!!!!!!!!!!!!!
/// Topology struct represents the whole CPUSocket architecture,
/// from the electricity consumption point of view,
/// including the potentially multiple CPUSocket sockets.
/// Owns a vector of CPUSocket structs representing each socket.
#[derive(Debug)]
pub struct Topology {
    pub sockets: Vec<CPUSocket>,
    pub remote: bool,
    pub cpuinfo: String,
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            sockets: vec![],
            cpuinfo: Topology::get_attributes_linux(),
            remote: false
        }
    }

    pub fn get_attributes_linux() -> String{
        let f = fs::read_to_string("/proc/cpuinfo");
        f.unwrap()
    }

    pub fn safe_add_socket(
        &mut self, socket_id: u16, domains: Vec<Domain>,
        attributes: Vec<Vec<HashMap<String, String>>>,
        counter_uj_path: String
    ) {
        let result: Vec<&CPUSocket> = self.sockets.iter().filter(|s| s.id == socket_id).collect();
        if result.len() == 0 {
            let socket = CPUSocket::new(
                socket_id, domains, attributes, counter_uj_path
            );
            self.sockets.push(socket);
        }
    }

    pub fn safe_add_domain_to_socket(&mut self, socket_id: u16, domain_id: u16, name: &str, uj_counter: &str) {
        let iterator = self.sockets.iter_mut();
        for socket in iterator {
            if socket.id == socket_id {
                socket.safe_add_domain(
                    Domain::new(
                        domain_id, String::from(name), String::from(uj_counter)
                    )
                );
            }
        }
    }
}

// !!!!!!!!!!!!!!!!! CPUSocket !!!!!!!!!!!!!!!!!!!!!!!
/// CPUSocket struct represents a CPU socket, owning CPU cores.
#[derive(Debug)]
pub struct CPUSocket {
    pub id: u16,
    pub domains: Vec<Domain>,
    pub attributes: Vec<Vec<HashMap<String, String>>>,
    pub counter_uj_path: String,
    last_counter_value_uj: String,
    current_counter_value_uj: String
}
impl RecordGenerator for CPUSocket {
    fn get_record(&self) -> Record {
        Record::new(
            SystemTime::now(),
            self.read_counter_uj().unwrap().parse().unwrap(),
            units::EnergyUnit::MicroJoule
        )
    }
}
impl CPUSocket {
    fn new(id: u16, domains: Vec<Domain>, attributes: Vec<Vec<HashMap<String, String>>>, counter_uj_path: String) -> CPUSocket {
        CPUSocket { id, domains, attributes, counter_uj_path, last_counter_value_uj: String::from(""), current_counter_value_uj: String::from("") }
    }

    fn safe_add_domain(&mut self, domain: Domain) {
        let result: Vec<&Domain> = self.domains.iter().filter(|d| d.id == domain.id).collect();
        if result.len() == 0 {
            self.domains.push(domain);
        }
    }

    pub fn read_counter_uj(&self) -> Result<String, Box<dyn Error>> {
        match fs::read_to_string(&self.counter_uj_path) {
            Ok(result) => Ok(result),
            Err(error) => Err(Box::new(error))
        }
    }
}

// !!!!!!!!!!!!!!!!! Domain !!!!!!!!!!!!!!!!!!!!!!!
/// Domain struct represents a part of a CPUSocket from the
/// electricity consumption point of view.
#[derive(Debug)]
pub struct Domain {
    pub id: u16,
    pub name: String,
    pub counter_uj_path: String
}
impl Domain {
    fn new(id: u16, name: String, counter_uj_path: String) -> Domain{
        Domain{ id, name, counter_uj_path }
    }
    pub fn read_counter_uj(&self) -> Result<String, Box<dyn Error>> {
        match fs::read_to_string(&self.counter_uj_path) {
            Ok(result) => Ok(result),
            Err(error) => Err(Box::new(error))
        }
    }
}
impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Domain: {}", self.name)
    }
}


// !!!!!!!!!!!!!!!!! Record !!!!!!!!!!!!!!!!!!!!!!!
/// Record struct represents an electricity consumption measurement
/// tied to a domain.
#[derive(Debug)]
pub struct Record{
    timestamp: SystemTime,
    value: i128,
    unit: units::EnergyUnit
}

impl Record {
    fn new(timestamp: SystemTime, value: i128, unit: units::EnergyUnit) -> Record{
        Record{ timestamp, value, unit }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recorded {} {}", self.value, self.unit)
    }
}

// !!!!!!!!!!!!!!!!! Sensor !!!!!!!!!!!!!!!!!!!!!!!
/// Sensor trait, the Sensor API.
pub trait Sensor {
    fn get_topology(&mut self) -> Box<Option<&Topology>>;
    //fn get_record(&self) -> Record;
    fn generate_topology (&self) -> Result<Topology, Box<dyn Error>>;
}