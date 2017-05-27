extern crate clap;
extern crate simplelog;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub type UdpPort = u16;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum VerbosityLevel {
    DEBUG = 0,
    INFO  = 1,
    WARN  = 2,
    ERROR = 3
}

impl Into<simplelog::LogLevelFilter> for VerbosityLevel {
    fn into(self) -> simplelog::LogLevelFilter {
        match self {
            VerbosityLevel::DEBUG => simplelog::LogLevelFilter::Debug,
            VerbosityLevel::INFO  => simplelog::LogLevelFilter::Info,
            VerbosityLevel::WARN  => simplelog::LogLevelFilter::Warn,
            VerbosityLevel::ERROR => simplelog::LogLevelFilter::Error,
        }
    }
}

#[derive(Debug, Clone)]
/// Holds the program's runtime configuration
pub struct RuntimeConfig {
    pub multicast_group: IpAddr,
    pub multicast_port:  UdpPort,
    pub http_bind:       String,
    pub ws_bind:         String,
    pub verbosity_level: VerbosityLevel
}

pub struct ArgsParser;

impl ArgsParser {
    fn to_ip_addr(o: Option<&str>) -> IpAddr {
        let default_ip = IpAddr::V4(Ipv4Addr::from_str("239.0.0.1").unwrap());
        match o {
            Some(ip_str) => {
                if Ipv6Addr::from_str(ip_str).is_ok() {
                    IpAddr::V6(Ipv6Addr::from_str(ip_str).unwrap())
                } else if Ipv4Addr::from_str(ip_str).is_ok() {
                    IpAddr::V4(Ipv4Addr::from_str(ip_str).unwrap())
                } else {
                    default_ip
                }
            },
            None => default_ip
        }
    }

    fn to_port(o: Option<&str>) -> UdpPort {
        let default_port = 8899;
        match o.unwrap_or(default_port.to_string().as_str()).parse::<u16>() {
            Ok(rv) => rv,
            Err(_) => default_port
        }
    }

    fn to_verbosity_level(occ: u64) -> VerbosityLevel {
        match occ {
            0   => VerbosityLevel::ERROR,
            1   => VerbosityLevel::WARN,
            2   => VerbosityLevel::INFO,
            3   => VerbosityLevel::DEBUG,
            _   => VerbosityLevel::DEBUG
        }
    }

    pub fn from_cli() -> RuntimeConfig {
        let matches = clap::App::new("sensorweb-NodeMCU-collector")
                              .version("0.1")
                              .author("<lissyx@lissyx.dyndns.org>")
                              .about("Network collector for sensorweb-NodeMCU: listens on IP multicast group and provides data over WebSocket.")
                              .arg(clap::Arg::with_name("mcast")
                                   .short("m")
                                   .long("mcast")
                                   .value_name("MCAST")
                                   .help("Multicast address")
                                   .takes_value(true)
                                   .required(false))
                              .arg(clap::Arg::with_name("port")
                                   .short("p")
                                   .long("port")
                                   .value_name("PORT")
                                   .help("Multicast port")
                                   .takes_value(true)
                                   .required(false))
                              .arg(clap::Arg::with_name("http_bind")
                                   .short("h")
                                   .long("http_bind")
                                   .value_name("HTTP_BIND")
                                   .help("IP:PORT to bind for HTTP")
                                   .takes_value(true)
                                   .required(false))
                              .arg(clap::Arg::with_name("ws_bind")
                                   .short("w")
                                   .long("ws_bind")
                                   .value_name("WS_BIND")
                                   .help("IP:PORT to bind for WebSocket")
                                   .takes_value(true)
                                   .required(false))
                              .arg(clap::Arg::with_name("v")
                                   .short("v")
                                   .multiple(true)
                                   .help("Sets the level of verbosity"))
                              .get_matches();


        RuntimeConfig {
            multicast_group: ArgsParser::to_ip_addr(matches.value_of("mcast")),
            multicast_port:  ArgsParser::to_port(matches.value_of("port")),
            http_bind: String::from(matches.value_of("http_bind").unwrap_or("0.0.0.0:8000")),
            ws_bind: String::from(matches.value_of("ws_bind").unwrap_or("0.0.0.0:8001")),
            verbosity_level: ArgsParser::to_verbosity_level(matches.occurrences_of("v"))
        }
    }
}

#[test]
fn test_to_ip_addr() {
    assert_eq!(ArgsParser::to_ip_addr(Some("")), IpAddr::V4(Ipv4Addr::new(239, 0, 0, 1)));
    assert_eq!(ArgsParser::to_ip_addr(Some("239.255.0.1")), IpAddr::V4(Ipv4Addr::new(239, 255, 0, 1)));
    assert_eq!(ArgsParser::to_ip_addr(Some("1.2.3.4")), IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)));
    assert_eq!(ArgsParser::to_ip_addr(Some("::1")), IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
    assert_eq!(ArgsParser::to_ip_addr(Some("ffx3::1")), IpAddr::V4(Ipv4Addr::new(239, 0, 0, 1)));
    assert_eq!(ArgsParser::to_ip_addr(Some("ff03::1")), IpAddr::V6(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 1)));
}

#[test]
fn test_to_port() {
    assert_eq!(ArgsParser::to_port(Some("xxx")), 8899);
    assert_eq!(ArgsParser::to_port(Some("8899")), 8899);
    assert_eq!(ArgsParser::to_port(Some("1234")), 1234);
}

#[test]
fn test_to_verbosity_level() {
    assert_eq!(ArgsParser::to_verbosity_level(0),  VerbosityLevel::ERROR);
    assert_eq!(ArgsParser::to_verbosity_level(1),  VerbosityLevel::WARN);
    assert_eq!(ArgsParser::to_verbosity_level(2),  VerbosityLevel::INFO);
    assert_eq!(ArgsParser::to_verbosity_level(3),  VerbosityLevel::DEBUG);
    assert_eq!(ArgsParser::to_verbosity_level(4),  VerbosityLevel::DEBUG);
    assert_eq!(ArgsParser::to_verbosity_level(42), VerbosityLevel::DEBUG);
}

#[test]
fn test_args() {
    let rc = ArgsParser::from_cli();

    assert_eq!(rc.multicast_group.to_string(), "239.0.0.1");
    assert_eq!(rc.multicast_port.to_string(), "8899");
    assert_eq!(rc.verbosity_level, VerbosityLevel::ERROR);
}
