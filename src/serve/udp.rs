use std::net::SocketAddr;

use mio::net::UdpSocket;

use super::ProtoBinder;

// todo(tmfink): finish handling UDP

impl ProtoBinder for UdpSocket {
    fn bind_proto(addr: SocketAddr) -> std::io::Result<Self> {
        Self::bind(addr)
    }
}
