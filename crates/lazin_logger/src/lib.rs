use std::sync::atomic::{AtomicBool, Ordering};

pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

enum Coloured {
    Yes,
    No,
}

impl Level {
    fn prefix(&self, coloured: Coloured) -> &'static str {
        match (self, coloured) {
            (Level::Trace, Coloured::No) => "TRACE",
            (Level::Debug, Coloured::No) => "DEBUG",
            (Level::Info, Coloured::No) => "INFO",
            (Level::Warn, Coloured::No) => "WARN",
            (Level::Error, Coloured::No) => "ERROR",

            (Level::Trace, Coloured::Yes) => "\x1b[36mTRACE\x1b[0m",
            (Level::Debug, Coloured::Yes) => "\x1b[32mDEBUG\x1b[0m",
            (Level::Info, Coloured::Yes) => "\x1b[34mINFO\x1b[0m",
            (Level::Warn, Coloured::Yes) => "\x1b[33mWARN\x1b[0m",
            (Level::Error, Coloured::Yes) => "\x1b[31mERROR\x1b[0m",
        }
    }
}

struct InnerLogger {
    coloured: AtomicBool,
}

impl InnerLogger {
    const fn new() -> Self {
        Self {
            coloured: AtomicBool::new(true),
        }
    }

    fn coloured(&self) -> Coloured {
        match self.coloured.load(Ordering::Relaxed) {
            true => Coloured::Yes,
            false => Coloured::No,
        }
    }

    fn log(&self, level: Level, message: &str) {
        println!("{}: {}", level.prefix(self.coloured()), message)
    }

    fn print(&self, message: &str) {
        println!("{}", message)
    }
}

pub struct LazinLogger(InnerLogger);
impl LazinLogger {
    pub fn log(&self, level: Level, message: &str) {
        self.0.log(level, message);
    }

    pub fn print(&self, message: &str) {
        self.0.print(message);
    }
}

static LOGGER: LazinLogger = LazinLogger(InnerLogger::new());

pub fn default_logger() -> &'static LazinLogger {
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
#[macro_export]
macro_rules! print{ ($($a:tt)*) => { $crate::default_logger().print(&format!($($a)*)) }; }

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
