extern crate log;
extern crate log4rs;
extern crate poston;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod fluentd;
pub use fluentd::*;
