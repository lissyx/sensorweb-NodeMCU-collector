use std::sync::mpsc::Receiver;

use message::NetworkMsg;

pub fn th_message_manager(rx: Receiver<NetworkMsg>) {
    info!("Message thread started");

    loop {
        info!("Waiting ...");
        match rx.recv() {
            Ok(parsed_msg) => info!("Received message: {:?}", parsed_msg),
            Err(err_recv)  => error!("Error trying to rx.recv(): {:?}", err_recv)
        }
    }
}
