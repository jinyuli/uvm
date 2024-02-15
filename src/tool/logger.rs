use std::path::Path;

use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::{
            policy::{
                self, compound::roll::fixed_window::FixedWindowRoller,
                compound::trigger::size::SizeTrigger,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

pub use log::{debug, error, info};

const LOGGER_SIZE: u64 = 10 * 1024 * 1024;
const LOGGER_FILE_COUNT: u32 = 10;

pub fn init_logger(log_path: &Path) -> Result<(), SetLoggerError> {
    let log_level = get_log_level();
    let rolling_file_path = log_path.join("uvm.{}.log");
    let rolling_file_str = rolling_file_path
        .as_os_str()
        .to_str()
        .expect("rolling file name error");

    let trigger = SizeTrigger::new(LOGGER_SIZE);
    let roller = FixedWindowRoller::builder()
        .build(rolling_file_str, LOGGER_FILE_COUNT)
        .unwrap();
    let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let rolling_file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}\n")))
        .build(rolling_file_path, Box::new(policy))
        .unwrap();

    let config = if is_release() {
        Config::builder()
            .appender(Appender::builder().build("roller", Box::new(rolling_file)))
            .logger(Logger::builder().build("uvm", log_level))
            .logger(Logger::builder().build("html5ever", LevelFilter::Info))
            .build(Root::builder().appender("roller").build(LevelFilter::Warn))
            .unwrap()
    } else {
        let stderr = ConsoleAppender::builder().target(Target::Stderr).build();
        Config::builder()
            .appender(Appender::builder().build("roller", Box::new(rolling_file)))
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .logger(Logger::builder().build("uvm", log_level))
            .logger(Logger::builder().build("html5ever", LevelFilter::Info))
            .build(
                Root::builder()
                    .appender("roller")
                    .appender("stderr")
                    .build(LevelFilter::Warn),
            )
            .unwrap()
    };

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config)?;

    Ok(())
}

#[cfg(build = "release")]
fn is_release() -> bool {
    true
}

#[cfg(not(build = "release"))]
fn is_release() -> bool {
    false
}

#[cfg(profile = "release")]
fn get_log_level() -> LevelFilter {
    LevelFilter::Info
}

#[cfg(not(profile = "release"))]
fn get_log_level() -> LevelFilter {
    LevelFilter::Debug
}
