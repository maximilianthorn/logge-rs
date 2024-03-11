//! A minimal logger inspired by simple_logging, with slighly more
//! customization and a different logging style.

use chrono;
use colored::Colorize;
use lazy_static::lazy_static;
use log::{Level, Metadata, Record};
use std::{
    io::{self, Write},
    sync::Mutex,
};

lazy_static! {
    static ref LOGGER: Logger = Logger {
        inner: Mutex::new(LoggerOptions::default()),
    };
}

struct Logger {
    inner: Mutex<LoggerOptions>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.inner.lock().unwrap().enabled)(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = chrono::Utc::now();
            let level = record.level().to_string();
            let level = match record.level() {
                Level::Error => level.red(),
                Level::Warn => level.yellow(),
                Level::Info => level.green(),
                Level::Debug => level.white(),
                Level::Trace => level.bright_black(),
            };

            let _ = write!(
                self.inner.lock().unwrap().writer,
                "[{}] {:5} [{}] - {}\n",
                now.format("%F %T").to_string().bold().bright_black(),
                level,
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

// /// Sets where the logs are written to.
// ///
// /// See [`LoggerOptions::set_writer`](LoggerOptions::set_writer)
// pub fn set_writer<T>(writer: T)
// where
//     T: Write + Send + 'static,
// {
//     LOGGER.inner.lock().unwrap().set_writer(writer);
// }

// /// Checks wether the current log
// /// shall be executed.
// ///
// /// See [`LoggerOptions::set_enabled`](LoggerOptions::set_enabled)
// pub fn set_enabled<T>(enabled: T)
// where
//     T: Fn(&Metadata) -> bool + Send + 'static,
// {
//     LOGGER.inner.lock().unwrap().set_enabled(enabled);
// }

// /// Apply `LoggerOptions` to the `Logger` and set it to the
// /// global logger.
// ///
// /// *Must only be called once!*
// pub fn with_options(options: LoggerOptions) {
//     *LOGGER.inner.lock().unwrap() = options;
//     log::set_logger(&*LOGGER).unwrap();
// }

pub struct LoggerOptions {
    writer: Box<dyn Write + Send + 'static>,
    enabled: Box<dyn Fn(&Metadata) -> bool + Send + 'static>,
}

impl Default for LoggerOptions {
    /// Uses the following defaults:
    ///
    /// ```rust
    /// LoggerOptions {
    ///     writer: io::stderr(),
    ///     enabled: |metadata| metadata.level() <= log::max_level(),
    /// }
    /// ```
    fn default() -> Self {
        LoggerOptions {
            writer: Box::new(io::stderr()),
            enabled: Box::new(|metadata| metadata.level() <= log::max_level()),
        }
    }
}

impl LoggerOptions {
    /// Start the `Logger` with the given `LoggerOptions`.
    ///
    /// *Must only be called once!*
    pub fn start(self) {
        *LOGGER.inner.lock().unwrap() = self;
        log::set_logger(&*LOGGER).unwrap();
    }

    /// Sets where the logs are written to.
    ///
    /// # Examples
    ///
    /// Log to a file.
    /// ```rust
    /// let file = std::fs::File::create("foo.log").unwrap();
    /// LoggerOptions::default().set_writer(file).start();
    /// ```
    ///
    /// Log to stdout.
    /// ```rust
    /// let stdout = std::io::stdout();
    /// ´LoggerOptions::default().set_writer(stdout).start();
    /// ```
    pub fn set_writer<T>(mut self, writer: T) -> Self
    where
        T: Write + Send + 'static,
    {
        self.writer = Box::new(writer);
        self
    }

    /// Checks wether the current log
    /// shall be executed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Checks for appropriate log level and only logs
    /// // if log is coming from our own crate.
    /// let options = LoggerOptions::default().set_enabled(|metadata| {
    ///     metadata.level() <= log::max_level()
    ///         && metadata.target().starts_with(env!("CARGO_PKG_NAME"))
    /// });
    ///
    /// options.start();
    /// ```
    pub fn set_enabled<T>(mut self, enabled: T) -> Self
    where
        T: Fn(&Metadata) -> bool + Send + 'static,
    {
        self.enabled = Box::new(enabled);
        self
    }
}
