use std::{
    // collections::HashMap,
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
const SERVER: Token = Token(0);

pub fn serve_forever(config: &Config) -> crate::Result<()> {
    // todo(tmfink): handle multiple services
    // todo(tmfink): handle UDP services
    let service = &config.services()[0];
    assert_eq!(service.socket_type, SocketType::Tcp);

    // Bind a server socket to connect to.
    let addr: SocketAddr = service.socket_addr()?;
    let mut tcp_listener = TcpListener::bind(addr).with_message("failed to open config")?;

    // Construct a new `Poll` handle as well as the `Events` we'll store into
    let mut poll = Poll::new().with_message("failed to create mio::Poll")?;
    let mut events = Events::with_capacity(EVENTS_CAPACITY);

    // Register the stream with `Poll`
    poll.registry()
        .register(&mut tcp_listener, SERVER, Interest::READABLE)
        .with_message("failed to registry mio listener")?;

    // Map of `Token` -> `TcpStream`.
    let mut child_procs: Vec<Child> = Vec::new();

    // Wait for the socket to become ready. This has to happens in a loop to
    // handle spurious wakeups.
    loop {
        poll.poll(&mut events, None)
            .with_message("mio poll failed")?;

        for event in &events {
            match event.token() {
                SERVER if event.is_readable() => loop {
                    match tcp_listener.accept() {
                        Ok((client_connection, client_addr)) => {
                            debug!(
                                "Got connection from {} for service {:?}",
                                client_addr, service
                            );
                            match handle_new_connection(client_connection, service) {
                                Ok(child) => {
                                    child_procs.push(child);
                                    trace!("child_procs: {:?}", &child_procs);
                                }
                                Err(err) => {
                                    error!("Failed to handle new connection: {}", err);
                                }
                            }
                        }
                        Err(ref err) if would_block(err) => break,
                        Err(err) => return Err(err.with_message("accept failed")),
                    }
                },
                _ => {}
            }
        }
    }
}

fn handle_new_connection(connection: TcpStream, service: &Service) -> crate::Result<Child> {
    let mut cmd = Command::new(&service.server);
    cmd.args(&service.server_args.0);
    unsafe {
        cmd.pre_exec(move || {
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
