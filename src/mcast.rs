use std::str;
use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};

use std::sync::mpsc::SyncSender;

use args::UdpPort;
use message::{NetworkMsg, parse_from_string};

fn bind_and_join(ip: IpAddr, port: UdpPort) -> UdpSocket {
    let bind_addr = format!("{}:{}", "0.0.0.0", port);
    info!("Using {:?}", bind_addr);

	let mut socket = match UdpSocket::bind(bind_addr) {
        Ok(s)  => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };
	info!("Bind: {:?}", socket);

    let mcast_join = match ip {
        IpAddr::V6(a) => socket.join_multicast_v6(&a, 0),
	    IpAddr::V4(a) => socket.join_multicast_v4(&a, &Ipv4Addr::new(0, 0, 0, 0)),
	};

    match mcast_join {
        Err(why) => error!("Join error: {:?}", why),
        Ok(_) => info!("Joined multicast group: {:?}", ip),
    };

    socket
}

fn leave(ip: IpAddr, port: UdpPort, socket: UdpSocket) {
    let mcast_leave = match ip {
        IpAddr::V6(a) => socket.leave_multicast_v6(&a, 0),
	    IpAddr::V4(a) => socket.leave_multicast_v4(&a, &Ipv4Addr::new(0, 0, 0, 0)),
    };

    match mcast_leave {
        Err(why) => error!("Leave error: {:?}", why),
        Ok(_) => info!("Left multicast group: {:?}", ip),
    };

    drop(socket)
}

fn read_to_string(socket: &UdpSocket) -> String {
    let mut buf = [0; 1024];

    match socket.recv_from(&mut buf) {
        Ok((received, src)) => {
            let s = String::from_str(str::from_utf8(&buf[0..received]).unwrap_or("").trim()).unwrap();
            debug!("received {} bytes from {}: {}", received, src, s);
            s
        }
        Err(e) => {
            debug!("recv function failed: {:?}", e);
            String::from("")
        }
    }
}

pub fn bind_mcast(ip: IpAddr, port: UdpPort, tx: SyncSender<NetworkMsg>) {
    info!("Network thread started");

    let socket = bind_and_join(ip, port);

    loop {
        let msg = parse_from_string(read_to_string(&socket));
        info!("Sending parsed message: {:?}", msg);
        match tx.send(msg) {
            Ok(_)    => debug!("Successfully sent message to thread"),
            Err(err) => error!("Error while sending message to thread: {:?}", err)
        }
    }

	leave(ip, port, socket)
}
