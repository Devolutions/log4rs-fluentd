extern crate log4rs;
extern crate log4rs_fluentd;
#[macro_use]
extern crate log;

use log4rs::append::console::ConsoleAppender;

fn main() {
    // Use custom PatternEncoder to keep only the log itself (no filename, timestamp...).
    let encoder = Box::new(log4rs::encode::pattern::PatternEncoder::new("{m}"));
    let stdout = ConsoleAppender::builder().build();
    let fluentd_appender = Box::new(
        log4rs_fluentd::FluentdAppender::builder()
            .encoder(encoder)
            .tag("manually.event")
            .build("127.0.0.1:24224".to_string()),
    );

    let config = log4rs::config::Config::builder()
        .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
        .appender(log4rs::config::Appender::builder().build("fluentd", fluentd_appender))
        .logger(
            log4rs::config::Logger::builder()
                .appender("fluentd")
                .additive(true)
                .build("manually", log::LevelFilter::Trace),
        )
        .build(log4rs::config::Root::builder().build(log::LevelFilter::Off))
        .unwrap();
    log4rs::init_config(config).unwrap();

    for i in 0..5 {
        trace!("Example trace message: {}", i);
        debug!("Example debug message: {}", i);
        info!("Example information message: {}", i);
        warn!("Example warning message: {}", i);
        error!("Example error message: {}", i);
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
    }

    ::std::thread::sleep(::std::time::Duration::from_secs(10));
    println!("Check your logs for new messages");
}
