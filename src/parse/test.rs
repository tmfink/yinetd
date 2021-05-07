use once_cell::sync::Lazy;

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

static DEFAULT_SERVICE: Lazy<Service> = Lazy::new(|| Service {
    name: "".to_string(),
    server: "".to_string(),
    port: 0,
    uid: None,
    server_args: None,
    socket_type: None,
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
        server_args: None,
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
                ..service_a
            },
            service_b
        ]
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
