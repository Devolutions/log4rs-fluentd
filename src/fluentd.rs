use log::Record;
use log4rs;
use log4rs::encode::writer::simple::SimpleWriter;
use poston::client::{Client, Settings, WorkerPool};
use std::error::Error;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Serialize)]
pub struct LogRecord {
    level: String,
    message: String,
    #[serde(skip_serializing)]
    time: SystemTime,
}

impl LogRecord {
    fn new(level: String, message: &str) -> Self {
        LogRecord {
            level,
            message: message.to_owned(),
            time: SystemTime::now(),
        }
    }
}

#[derive(Debug)]
pub struct FluentdAppender {
    encoder: Box<log4rs::encode::Encode>,
    sender: Mutex<Sender<LogRecord>>,
}

impl FluentdAppender {
    pub fn builder() -> FluentdAppenderBuilder {
        FluentdAppenderBuilder {
            encoder: None,
            tag: "".to_owned(),
        }
    }
}

impl ::log4rs::append::Append for FluentdAppender {
    fn append(&self, record: &Record) -> Result<(), Box<Error + Sync + Send>> {
        let mut writer = SimpleWriter(Vec::<u8>::new());
        self.encoder.encode(&mut writer, record)?;

        let log_record = LogRecord::new(format!("{}", record.level()), &String::from_utf8_lossy(&writer.0));
        let sender = self.sender.lock().unwrap();
        sender.send(log_record)?;
        Ok(())
    }

    fn flush(&self) {}
}

/// Builder for `FluentdAppender`.
pub struct FluentdAppenderBuilder {
    encoder: Option<Box<log4rs::encode::Encode>>,
    tag: String,
}

impl FluentdAppenderBuilder {
    /// Set custom encoder.
    pub fn encoder(mut self, encoder: Box<log4rs::encode::Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = tag.to_owned();
        self
    }
    /// Consume builder and produce `FluentdAppender`.
    pub fn build<A>(self, addr: A) -> FluentdAppender
    where
        A: ToSocketAddrs + Clone,
        A: Send + 'static,
    {
        let (sender, receiver): (Sender<LogRecord>, Receiver<LogRecord>) = ::std::sync::mpsc::channel();
        let tag_clone = self.tag.clone();

        //Thread receiving all log_record and sending them to fluentd
        ::std::thread::spawn(move || {
            let settings = Settings {
                connection_retry_timeout: ::std::time::Duration::from_secs(5),
                ..Default::default()
            };

            match WorkerPool::with_settings(&addr, &settings) {
                Ok(pool) => loop {
                    match receiver.recv() {
                        Ok(log_record) => {
                            if let Err(e) = pool.send(tag_clone.clone(), &log_record, log_record.time) {
                                println!("Log record can't be sent to fluentd: {}", e);
                            }
                        }
                        Err(e) => {
                            println!("Can't receive new log record: {}", e);
                            break;
                        }
                    }
                },

                Err(e) => {
                    println!("Fluentd worker pool can't be created: {}", e);
                }
            };
        });

        FluentdAppender {
            encoder: self.encoder
                .unwrap_or_else(|| Box::new(log4rs::encode::pattern::PatternEncoder::default())),
            sender: Mutex::new(sender),
        }
    }
}
