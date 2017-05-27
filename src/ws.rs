extern crate websocket;

use std::thread;
use self::websocket::OwnedMessage;
use self::websocket::sync::Server;

fn ws_handler() {

}

pub fn th_ws_listener(ws_bind: String) {
    info!("WebSocket thread started: {}", ws_bind);

	let server = Server::bind(ws_bind).unwrap();

	for request in server.filter_map(Result::ok) {
        debug!("Accepted one connection!");
		// Spawn a new thread for each connection.
		thread::spawn(move || {
            debug!("Checking protocol");
			if !request.protocols().contains(&"rust-websocket".to_string()) {
                debug!("Protocol is not acceptable.");
				request.reject().unwrap();
				return;
			}

			let mut client = request.use_protocol("rust-websocket").accept().unwrap();

			let ip = client.peer_addr().unwrap();

			println!("Connection from {}", ip);

			let message = OwnedMessage::Text("Hello".to_string());
			client.send_message(&message).unwrap();

			let (mut receiver, mut sender) = client.split().unwrap();

			for message in receiver.incoming_messages() {
				let message = message.unwrap();

				match message {
					OwnedMessage::Close(_) => {
						let message = OwnedMessage::Close(None);
						sender.send_message(&message).unwrap();
						println!("Client {} disconnected", ip);
						return;
					}
					OwnedMessage::Ping(ping) => {
						let message = OwnedMessage::Pong(ping);
						sender.send_message(&message).unwrap();
					}
					_ => sender.send_message(&message).unwrap(),
				}
			}
		});
	}
}
