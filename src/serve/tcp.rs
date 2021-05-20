use std::net::SocketAddr;

use log::{debug, error};
use mio::net::TcpListener;

use super::{
    create_server_state, handle_new_connection, try_reap_children, would_block, ProtoBinder,
    ProtoServerState, MAX_WAIT,
};
use crate::{config::Config, error::StdIoErrorExt};

impl ProtoBinder for TcpListener {
    fn bind_proto(addr: SocketAddr) -> std::io::Result<Self> {
        Self::bind(addr)
    }
}

pub fn serve_tcp_forever(config: Config) -> crate::Result<()> {
    let ProtoServerState {
        mut service_states,
        mut poll,
        mut events,
    }: ProtoServerState<TcpListener> = create_server_state(&config)?;

    loop {
        poll.poll(&mut events, Some(MAX_WAIT))
            .with_message("mio poll failed")?;

        try_reap_children(&mut service_states);

        for event in &events {
            let service_state = &mut service_states[event.token().0];
            if !event.is_readable() {
                continue;
            }
            loop {
                let (client_connection, client_addr) = match service_state.proto_binder.accept() {
                    Ok(res) => res,
                    Err(ref err) if would_block(err) => break,
                    Err(err) => return Err(err.with_message("accept failed")),
                };

                debug!(
                    "Got connection from {} for service {:?}",
                    client_addr, service_state.service.name
                );
                match handle_new_connection(client_connection, &service_state.service) {
                    Ok(child) => {
                        service_state.add_child(child);
                    }
                    Err(err) => {
                        error!("Failed to handle new connection: {}", err);
                    }
                }
            }
        }
    }
}
