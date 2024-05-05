use crate::CLI;
use log::{warn, LevelFilter, Metadata, Record};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub static LOGGER: SimpleLogger = SimpleLogger;

pub fn setup_logger(cli: &CLI) {
    // TODO: support custom loggers
    log::set_logger(&LOGGER).unwrap_or(());
    match cli.verbose {
        i8::MIN => log::set_max_level(LevelFilter::Off),
        -1 => log::set_max_level(LevelFilter::Error),
        0 => log::set_max_level(LevelFilter::Warn),
        1 => log::set_max_level(LevelFilter::Info),
        2 => log::set_max_level(LevelFilter::Debug),
        3 => log::set_max_level(LevelFilter::Trace),
        _ => {
            log::set_max_level(LevelFilter::Warn);
            warn!("Invalid `cli.verbose`: {}, default is Warn", cli.verbose);
        }
    }
}
