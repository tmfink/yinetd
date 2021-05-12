use std::{
    io,
    net::SocketAddr,
    os::unix::{io::AsRawFd, process::CommandExt},
    process::{Child, Command},
};

use log::{debug, error, trace};
use mio::{
    event::Events,
    net::{TcpListener, TcpStream},
    Interest, Poll, Token,
};
use nix::unistd::dup2;

use crate::{config::Config, config_types::SocketType, error::StdIoErrorExt, service::Service};

const EVENTS_CAPACITY: usize = 1024;

struct ServiceState<'a> {
    child_procs: Vec<Child>,
    service: &'a Service,
    tcp_listener: TcpListener,
}

pub struct ServerState<'a> {
    /// Map token index to service
    service_states: Vec<ServiceState<'a>>,
    poll: Poll,
    events: Events,
}

pub fn create_server_state(config: &Config) -> crate::Result<ServerState> {
    let poll = Poll::new().with_message("failed to create mio::Poll")?;
    let events = Events::with_capacity(EVENTS_CAPACITY);

    let mut service_states = Vec::new();

    for service in config.services() {
        assert_eq!(service.socket_type, SocketType::Tcp);

        let addr: SocketAddr = service.socket_addr()?;
        let mut tcp_listener = TcpListener::bind(addr).with_message("failed to open config")?;
        // Use index in service state as the token
        let token = Token(service_states.len());

        poll.registry()
            .register(&mut tcp_listener, token, Interest::READABLE)
            .with_message(format!(
                "failed to register service {:?} with mio",
                service.name
            ))?;

        let child_procs = Vec::new();
        service_states.push(ServiceState {
            child_procs,
            service,
            tcp_listener,
        })
    }

    Ok(ServerState {
        service_states,
        poll,
        events,
    })
}

pub fn serve_forever(config: Config) -> crate::Result<()> {
    // todo(tmfink): handle multiple services
    // todo(tmfink): handle UDP services

    let ServerState {
        mut service_states,
        mut poll,
        mut events,
    } = create_server_state(&config)?;

    loop {
        poll.poll(&mut events, None)
            .with_message("mio poll failed")?;

        for event in &events {
            let service_state = &mut service_states[event.token().0];
            if !event.is_readable() {
                continue;
            }
            loop {
                let (client_connection, client_addr) = match service_state.tcp_listener.accept() {
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
                        service_state.child_procs.push(child);
                        trace!("child_procs: {:?}", &service_state.child_procs);
                    }
                    Err(err) => {
                        error!("Failed to handle new connection: {}", err);
                    }
                }
            }
        }
    }
}

fn handle_new_connection(connection: TcpStream, service: &Service) -> crate::Result<Child> {
    let mut cmd = Command::new(&service.server);
    cmd.args(&service.server_args.0);
    unsafe {
        cmd.pre_exec(move || {
            // dup the socket with stdin/stdout

            trace!("in child");
            let sock_fd = connection.as_raw_fd();

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
