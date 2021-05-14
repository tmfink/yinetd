use std::{
    io,
    net::SocketAddr,
    os::unix::{io::AsRawFd, process::CommandExt},
    process::{Child, Command},
};

use log::trace;
use mio::{event::Events, Interest, Poll, Token};
use nix::unistd::dup2;

use crate::{config::Config, config_types::SocketType, error::StdIoErrorExt, service::Service};

mod tcp;
mod udp;

const EVENTS_CAPACITY: usize = 1024;

trait ProtoBinder: mio::event::Source + AsRawFd + Sized {
    fn bind_proto(addr: SocketAddr) -> std::io::Result<Self>;
}

struct ServiceState<'a, P: ProtoBinder> {
    // todo(tmfink): reap child processes
    child_procs: Vec<Child>,
    service: &'a Service,
    proto_binder: P,
}

struct ProtoServerState<'a, P: ProtoBinder> {
    /// Map token index to service
    service_states: Vec<ServiceState<'a, P>>,
    poll: Poll,
    events: Events,
}

fn create_server_state<P: ProtoBinder>(config: &Config) -> crate::Result<ProtoServerState<P>> {
    let poll = Poll::new().with_message("failed to create mio::Poll")?;
    let events = Events::with_capacity(EVENTS_CAPACITY);

    let mut service_states = Vec::new();

    for service in config.services() {
        assert_eq!(service.socket_type, SocketType::Tcp);

        let addr: SocketAddr = service.socket_addr()?;
        let mut proto_binder = P::bind_proto(addr).with_message("failed to open config")?;
        // Use index in service state as the token
        let token = Token(service_states.len());

        poll.registry()
            .register(&mut proto_binder, token, Interest::READABLE)
            .with_message(format!(
                "failed to register service {:?} with mio",
                service.name
            ))?;

        let child_procs = Vec::new();
        service_states.push(ServiceState {
            child_procs,
            service,
            proto_binder,
        })
    }

    Ok(ProtoServerState {
        service_states,
        poll,
        events,
    })
}

pub fn serve_forever(config: Config) -> crate::Result<()> {
    tcp::serve_tcp_forever(config)
    // todo(tmfink): handle other protocols
}

fn handle_new_connection<C: AsRawFd>(connection: C, service: &Service) -> crate::Result<Child> {
    let sock_fd = connection.as_raw_fd();

    let mut cmd = Command::new(&service.server);
    cmd.args(&service.server_args.0);
    unsafe {
        cmd.pre_exec(move || {
            // dup the socket with stdin/stdout

            trace!("in child");

            dup2(sock_fd, io::stdin().as_raw_fd()).expect("Failed to dup2() stdin in child");
            trace!("dup'd child stdin");

            dup2(sock_fd, io::stdout().as_raw_fd()).expect("Failed to dup2() stdout in child");
            trace!("dup'd child stdout");

            Ok(())
        });
    }
    // todo(tmfink): why does this fail?
    let child = cmd.spawn().with_message("failed to spawn child process")?;

    Ok(child)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}
