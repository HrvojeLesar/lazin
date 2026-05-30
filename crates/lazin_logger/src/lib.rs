use std::fmt::Display;

pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn prefix(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix())
    }
}

struct InnerLogger;
impl InnerLogger {
    fn log(&self, level: Level, message: &str) {
        println!("{level}: {message}")
    }
}

pub struct Logger(InnerLogger);
impl Logger {
    pub fn log(&self, level: Level, message: &str) {
        self.0.log(level, message);
    }
}

static LOGGER: Logger = Logger(InnerLogger);

pub fn default_logger() -> &'static Logger {
    &LOGGER
}

#[macro_export]
macro_rules! trace{ ($($a:tt)*) => { $crate::default_logger().log($crate::Level::Trace,  &format!($($a)*)) }; }
#[macro_export]
macro_rules! debug{ ($($a:tt)*) => { $crate::default_logger().log($crate::Level::Debug,  &format!($($a)*)) }; }
#[macro_export]
macro_rules! info{ ($($a:tt)*) => { $crate::default_logger().log($crate::Level::Info,  &format!($($a)*)) }; }
#[macro_export]
macro_rules! warn{ ($($a:tt)*) => { $crate::default_logger().log($crate::Level::Warn,  &format!($($a)*)) }; }
#[macro_export]
macro_rules! error{ ($($a:tt)*) => { $crate::default_logger().log($crate::Level::Error,  &format!($($a)*)) }; }

#[cfg(test)]
mod test {
    #[test]
    fn compile_and_log() {
        trace!("log");
        debug!("log");
        info!("log");
        warn!("log");
        error!("log");
    }
}
