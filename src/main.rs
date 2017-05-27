mod args;
use args::ArgsParser;

mod http;
use http::th_http_listener;

mod mcast;
use mcast::bind_mcast;

mod message;

mod message_manager;
use message_manager::th_message_manager;

mod ws;
use ws::th_ws_listener;

use std::sync::mpsc::sync_channel;
use std::thread;

#[macro_use]
extern crate log;
extern crate simplelog;

fn main() {
    let rc = ArgsParser::from_cli();

    let log_level = rc.verbosity_level.into();
    let _ = simplelog::TermLogger::init(log_level, simplelog::Config::default());

    debug!("Parsed all CLI args: {:?}", rc);

    let (tx, rx) = sync_channel(0);

    let mut threads = Vec::new();
    let thread_messages = thread::Builder::new().name("MessageManager".to_string()).spawn(move || {
        th_message_manager(rx);
    });
    threads.push(thread_messages);

    let rc_http = rc.clone();
    let thread_http = thread::Builder::new().name("HttpService".to_string()).spawn(move || {
        th_http_listener(rc_http.http_bind);
    });
    threads.push(thread_http);

	let rc_ws = rc.clone();
    let thread_ws = thread::Builder::new().name("WebSocketService".to_string()).spawn(move || {
        th_ws_listener(rc_ws.ws_bind);
    });
    threads.push(thread_ws);

    let thread_network = thread::Builder::new().name("MulticastListener".to_string()).spawn(move || {
        bind_mcast(rc.multicast_group.clone(), rc.multicast_port.clone(), tx);
    });
    threads.push(thread_network);

    for hdl in threads {
        if hdl.is_ok() {
            hdl.unwrap().join().unwrap();
        }
    }
}
