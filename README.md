# yinetd

[![Crates.io Badge](https://img.shields.io/crates/v/yinetd.svg)](https://crates.io/crates/yinetd)

An [inetd][inetd] implementation in the [Rust language][rust-lang].

# Stability

*Caution*:
This project is under development and absolutely not ready for production.

[inetd]: https://en.wikipedia.org/wiki/Inetd
[rust-lang]: https://www.rust-lang.org/

# Todo

- Protocols
    - [X] TCP
    - [ ] UDP
    - [ ] Unix sockets
- Config
    - [X] server
    - [X] server_args
    - [X] port
    - [X] socket_type
    - [X] inet_type (IPv4/IPv6)
    - [X] listen_ip
        - [ ] handle multiple interfaces
    - [ ] user
    - [ ] group
    - [ ] stderr behavior: dup, redirect, ignore
    - [ ] logging
    - [ ] nice level
    - [ ] env
    - [ ] rate_limit
    - [ ] connection_limit (instances)
    - [ ] umask
    - [ ] wait (single-threaded vs. multi-threaded)
    - [ ] include (other config files)


# License

Available under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) licenses.
