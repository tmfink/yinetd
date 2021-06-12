use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use once_cell::sync::Lazy;

use crate::{
    config_types::{InetType, SocketType},
    Error,
};

use super::*;

const PASS_NO_DEFAULT: &str = r#"
service service_a
{
    server = /usr/sbin/service-a
    port = 1234
}
"#;

const PASS_WHITESPACE: &str = "
default {
    \tuid =    42
}

service \t\t\tservice_a

{
    server = /usr/sbin/service-a
    port = \t\t1234
}


";

const PASS_WITH_DEFAULT_UID: &str = r#"
default {
    uid = 42
}

service service_a
{
    server = /usr/sbin/service-a
    port = 1234
}
"#;

const PASS_WITH_DEFAULT_PORT: &str = r#"
default {
    port = 1234
}

service service_a
{
    uid = 42
    server = /usr/sbin/service-a
}
"#;

const PASS_WITH_DEFAULT_OVERRIDE_UID: &str = r#"
default {
    uid = 42
}

service service_a
{
    server = /usr/sbin/service-a
    port = 1234
    uid = 50
}
"#;

const PASS_WITH_DEFAULT_OVERRIDE_PORT_UID: &str = r#"
default {
    uid = 42
    port = 2345
}

service service_a
{
    server = /usr/sbin/service-a
    port = 1234
    uid = 50
}
"#;

const PASS_WITH_MULTI_SERVICE1: &str = r#"
default {
    uid = 42
}

service service_a
{
    server = /usr/sbin/service-a
    port = 1234
}

service service_b
{
    server = /usr/sbin/service-b
    port = 5678
    uid = 0
}
"#;

const PASS_WITH_MULTI_SERVICE2: &str = r#"
default {
    uid = 42
    port = 39847
}

service service_a
{
    server = /usr/sbin/service-a
}

service service_b
{
    server = /usr/sbin/service-b
    port = 5678
    uid = 0
}
"#;

const PASS_SOCKET_DEFAULT_UDP: &str = r#"
default {
    uid = 42
    socket_type = udp
}

service service_a
{
    server = /usr/sbin/service-a
    port = 1234
}

service service_b
{
    server = /usr/sbin/service-b
    port = 5678
    socket_type = tcp
    uid = 0
}
"#;

const PASS_DEFAULT_ADDRESS_IPV4: &str = r#"
service service_a
{
    server = server
    port = 1234
    inet_type = ipv4
}
"#;

const PASS_DEFAULT_ADDRESS_IPV6: &str = r#"
service service_a
{
    server = server
    port = 1234
    inet_type = ipv6
}
"#;

const FAIL_MISSING_REQUIRED_OPTION: &str = r#"
service service_a
{
    port = 1234
    uid = 398
    server = server
}

service service_b
{
    port = 1234
    uid = 398
}
"#;

const FAIL_DUPLICATE_SERVICE: &str = r#"
service service_a
{
    port = 1234
    uid = 398
    server = server
}

service service_a
{
    port = 1234
    uid = 398
    server = another-server
}
"#;

const FAIL_DUPLICATE_OPTIONS_PORT: &str = r#"
service service_a
{
    port = 1234
    uid = 398
    server = server
}

service service_b
{
    port = 1234
    port = 456
    uid = 398
    server = another-server
}
"#;

const FAIL_DUPLICATE_OPTIONS_SERVER: &str = r#"
service service_a
{
    port = 1234
    uid = 398
    server = server
}

service service_b
{
    port = 1234
    uid = 398
    server = another-server
    server = another-server2
}
"#;

const FAIL_BAD_OPTION_PORT: &str = r#"
service service_a
{
    port = 1234abc
    server = server
}
"#;

const FAIL_BAD_OPTION_UID: &str = r#"
service service_a
{
    port = 1234
    server = server
    uid = zero
}
"#;

const FAIL_MISMACH_EXPECT_IPV4: &str = r#"
service service_a
{
    server = server
    port = 1234
    listen_address = fe80::1
    inet_type = ipv4
}
"#;

const FAIL_MISMACH_EXPECT_IPV6: &str = r#"
service service_a
{
    server = server
    port = 1234
    listen_address = 127.0.0.1
    inet_type = ipv6
}
"#;

static DEFAULT_SERVICE: Lazy<Service> = Lazy::new(|| Service {
    name: "".to_string(),
    server: "".to_string(),
    port: 0,
    uid: None,
    server_args: Default::default(),
    inet_type: InetType::Ipv4,
    socket_type: SocketType::Tcp,
    listen_address: None,
});

#[test]
fn config_no_default() {
    assert_eq!(
        parse_config_str(PASS_NO_DEFAULT).unwrap().services(),
        &[Service {
            name: "service_a".to_string(),
            server: "/usr/sbin/service-a".to_string(),
            port: 1234,
            ..DEFAULT_SERVICE.clone()
        }]
    );
}

#[test]
fn config_default() {
    let services = &[Service {
        name: "service_a".to_string(),
        server: "/usr/sbin/service-a".to_string(),
        port: 1234,
        uid: Some(42),
        ..DEFAULT_SERVICE.clone()
    }];
    assert_eq!(
        parse_config_str(PASS_WITH_DEFAULT_UID).unwrap().services(),
        services
    );
    assert_eq!(
        parse_config_str(PASS_WITH_DEFAULT_PORT).unwrap().services(),
        services
    );
    assert_eq!(
        parse_config_str(PASS_WHITESPACE).unwrap().services(),
        services
    );
}

#[test]
fn config_default_override() {
    let services = &[Service {
        name: "service_a".to_string(),
        server: "/usr/sbin/service-a".to_string(),
        port: 1234,
        uid: Some(50),
        ..DEFAULT_SERVICE.clone()
    }];
    assert_eq!(
        parse_config_str(PASS_WITH_DEFAULT_OVERRIDE_UID)
            .unwrap()
            .services(),
        services
    );
    assert_eq!(
        parse_config_str(PASS_WITH_DEFAULT_OVERRIDE_PORT_UID)
            .unwrap()
            .services(),
        services
    );
}

#[test]
fn config_empty() {
    assert_eq!(parse_config_str("").unwrap().services(), &[]);
}

#[test]
fn config_multi() {
    let service_a = Service {
        name: "service_a".to_string(),
        server: "/usr/sbin/service-a".to_string(),
        port: 1234,
        uid: Some(42),
        ..DEFAULT_SERVICE.clone()
    };
    let service_b = Service {
        name: "service_b".to_string(),
        server: "/usr/sbin/service-b".to_string(),
        port: 5678,
        uid: Some(0),
        ..DEFAULT_SERVICE.clone()
    };
    assert_eq!(
        parse_config_str(PASS_WITH_MULTI_SERVICE1)
            .unwrap()
            .services(),
        &[service_a.clone(), service_b.clone()]
    );
    assert_eq!(
        parse_config_str(PASS_WITH_MULTI_SERVICE2)
            .unwrap()
            .services(),
        &[
            Service {
                port: 39847,
                ..service_a.clone()
            },
            service_b.clone()
        ]
    );
    assert_eq!(
        parse_config_str(PASS_SOCKET_DEFAULT_UDP)
            .unwrap()
            .services(),
        &[
            Service {
                socket_type: SocketType::Udp,
                ..service_a
            },
            service_b
        ]
    );
}

#[test]
fn default_listen_addr() {
    let config_ipv4 = parse_config_str(PASS_DEFAULT_ADDRESS_IPV4).unwrap();
    let service_ipv4 = &config_ipv4.services()[0];
    assert_eq!(
        service_ipv4.socket_addr().unwrap().ip(),
        Ipv4Addr::UNSPECIFIED
    );

    let config_ipv6 = parse_config_str(PASS_DEFAULT_ADDRESS_IPV6).unwrap();
    let service_ipv6 = &config_ipv6.services()[0];
    assert_eq!(
        service_ipv6.socket_addr().unwrap().ip(),
        Ipv6Addr::UNSPECIFIED
    );
}

#[test]
fn config_missing_required_option() {
    let err = parse_config_str(FAIL_MISSING_REQUIRED_OPTION).unwrap_err();
    match err {
        crate::Error::MissingRequiredOption {
            context: _,
            option,
            service: _,
        } => assert_eq!(&option, "server"),
        _ => panic!("wrong error: {}", err),
    }
}
#[test]
fn config_duplicate_service() {
    let err = parse_config_str(FAIL_DUPLICATE_SERVICE).unwrap_err();
    match err {
        crate::Error::DuplicateService {
            context: _,
            service,
        } => assert_eq!(&service, "service_a"),
        _ => panic!("wrong error: {}", err),
    }
}

#[test]
fn config_duplicate_option() {
    let err = parse_config_str(FAIL_DUPLICATE_OPTIONS_PORT).unwrap_err();
    match err {
        crate::Error::DuplicateOption { context: _, option } => assert_eq!(&option, "port"),
        _ => panic!("wrong error: {}", err),
    }

    let err = parse_config_str(FAIL_DUPLICATE_OPTIONS_SERVER).unwrap_err();
    match err {
        crate::Error::DuplicateOption { context: _, option } => assert_eq!(&option, "server"),
        _ => panic!("wrong error: {}", err),
    }
}

#[test]
fn config_invalid_option() {
    let err = parse_config_str(FAIL_BAD_OPTION_PORT).unwrap_err();
    match err {
        crate::Error::OptionValueParse { context: _, option } => assert_eq!(&option, "port"),
        _ => panic!("wrong error: {}", err),
    }

    let err = parse_config_str(FAIL_BAD_OPTION_UID).unwrap_err();
    match err {
        crate::Error::OptionValueParse { context: _, option } => assert_eq!(&option, "uid"),
        _ => panic!("wrong error: {}", err),
    }
}

#[test]
fn mismatch_inet_type() {
    let expected_service_name = "service_a";

    let config = &parse_config_str(FAIL_MISMACH_EXPECT_IPV4).unwrap();
    let service = &config.services()[0];
    let err = service.socket_addr().unwrap_err();
    match err {
        Error::InetVersionAddressMismatch {
            expected_type,
            addr,
            service_name,
        } => {
            assert_eq!(expected_type, InetType::Ipv4);
            assert_eq!(addr, "fe80::1".parse::<IpAddr>().unwrap());
            assert_eq!(service_name, expected_service_name);
        }
        _ => panic!("wrong error: {}", err),
    }

    let config = &parse_config_str(FAIL_MISMACH_EXPECT_IPV6).unwrap();
    let service = &config.services()[0];
    dbg!(service);
    let err = service.socket_addr().unwrap_err();
    match err {
        Error::InetVersionAddressMismatch {
            expected_type,
            addr,
            service_name,
        } => {
            assert_eq!(expected_type, InetType::Ipv6);
            assert_eq!(addr, "127.0.0.1".parse::<IpAddr>().unwrap());
            assert_eq!(service_name, expected_service_name);
        }
        _ => panic!("wrong error: {}", err),
    }
}

// todo(tmfink): test re-used ports
