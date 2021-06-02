use std::{
    io,
    net::SocketAddr,
    os::unix::{io::AsRawFd, process::CommandExt},
    process::{Child, Command},
    time::Duration,
};

use log::trace;
use mio::{event::Events, Interest, Poll, Token};
use nix::unistd::dup2;

use crate::{config::Config, config_types::SocketType, error::StdIoErrorExt, service::Service};

mod service_state;
mod tcp;
mod udp;

use service_state::ServiceState;

const EVENTS_CAPACITY: usize = 1024;
const MAX_WAIT: Duration = Duration::from_millis(100);

pub(crate) trait ProtoBinder: mio::event::Source + AsRawFd + Sized {
    fn bind_proto(addr: SocketAddr) -> std::io::Result<Self>;
}

struct ProtoServerState<'a, P: ProtoBinder> {
    /// Map token index to service
    service_states: Vec<ServiceState<'a, P>>,
    poll: Poll,
    events: Events,
}

pub(crate) fn try_reap_children<P: ProtoBinder>(service_states: &mut Vec<ServiceState<'_, P>>) {
    for service_state in service_states.iter_mut() {
        service_state.try_reap_children();
    }
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

        service_states.push(ServiceState::new(service, proto_binder));
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

fn make_fd_blocking(fd: libc::c_int) {
    let fd_flag_bits = nix::fcntl::fcntl(fd, nix::fcntl::FcntlArg::F_GETFL).unwrap();
    let mut fd_flags = nix::fcntl::OFlag::from_bits(fd_flag_bits).unwrap();
    trace!("fd {} old flags: {:?}", fd, fd_flags);

    fd_flags.remove(nix::fcntl::OFlag::O_NONBLOCK);
    trace!("fd {} new flags: {:?}", fd, fd_flags);

    nix::fcntl::fcntl(fd, nix::fcntl::FcntlArg::F_SETFL(fd_flags)).unwrap();
}

fn handle_new_connection<C: AsRawFd>(connection: C, service: &Service) -> crate::Result<Child> {
    let sock_fd = connection.as_raw_fd();

    let mut cmd = Command::new(&service.server);
    cmd.args(&service.server_args.0);
    unsafe {
        cmd.pre_exec(move || {
            trace!("in child");

            // dup stdin/out/err to socket
            for &fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO].iter() {
                if let Err(err) = dup2(sock_fd, fd) {
                    panic!("Failed to dup2() fd {} as socket fd: {}", fd, err);
                }
                trace!("dup'd child fd {} to socket fd", fd);

                // after duping the socket, the fd will inherit non-blocking from the listener socket
                make_fd_blocking(fd);
            }

            Ok(())
        });
    }
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Err(err.with_message(format!(
                "failed to spawn child process executable {:?}",
                service.server
            )))
        }
    };

    Ok(child)
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}
