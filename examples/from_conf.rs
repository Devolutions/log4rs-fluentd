extern crate log4rs;
extern crate log4rs_fluentd;
#[macro_use]
extern crate log;
extern crate tempfile;

fn main() {
    use std::io::Write;

    let mut deserializers = log4rs::file::Deserializers::new();
    log4rs_fluentd::register(&mut deserializers);

    let yaml_conf = br#"
appenders:
  fluentd:
    kind: fluentd
    tag: from_conf.event
    addr: 127.0.0.1:24224
    encoder:
      pattern: "{m}"
root:
  level: trace
  appenders:
    - fluentd
loggers:
  poston:
    level: off
"#;
    // Note that configuration file should have right extension, otherwise log4rs will fail to recognize format.
    let mut tmp_conf = tempfile::Builder::new()
        .suffix(".yaml")
        .tempfile()
        .unwrap();
    tmp_conf.write_all(yaml_conf).unwrap();
    tmp_conf.flush().unwrap();

    log4rs::init_file(tmp_conf.path(), deserializers).unwrap();

    for i in 0..5 {
        trace!("Example trace message: {}", i);
        debug!("Example debug message: {}", i);
        info!("Example information message: {}", i);
        warn!("Example warning message: {}", i);
        error!("Example error message: {}", i);
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    }

    println!("Check your logs for new messages");
}
