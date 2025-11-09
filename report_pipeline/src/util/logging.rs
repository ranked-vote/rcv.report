use colored::*;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" | "warning" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }

    pub fn from_env() -> Self {
        env::var("RANKED_VOTE_LOG_LEVEL")
            .map(|s| Self::from_str(&s))
            .unwrap_or(LogLevel::Warn)
    }
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LogLevel::from_env(),
        }
    }

    pub fn error(&self, msg: &str) {
        if self.level >= LogLevel::Error {
            eprintln!("{}", msg.red());
        }
    }

    pub fn warn(&self, msg: &str) {
        if self.level >= LogLevel::Warn {
            eprintln!("{}", msg.yellow());
        }
    }

    pub fn info(&self, msg: &str) {
        if self.level >= LogLevel::Info {
            eprintln!("{}", msg);
        }
    }

    pub fn debug(&self, msg: &str) {
        if self.level >= LogLevel::Debug {
            eprintln!("{}", msg.bright_black());
        }
    }

    pub fn trace(&self, msg: &str) {
        if self.level >= LogLevel::Trace {
            eprintln!("{}", msg.bright_black());
        }
    }

    pub fn race(&self, jurisdiction: &str, election: &str, office: &str) {
        if self.level >= LogLevel::Warn {
            eprintln!(
                "{} {} - {} - {}",
                "🏁".green(),
                jurisdiction.bright_cyan(),
                election.bright_cyan(),
                office.bright_cyan()
            );
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref LOG: Logger = Logger::new();
}

// Convenience macros
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::util::LOG.error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::util::LOG.warn(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::util::LOG.info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::util::LOG.debug(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::util::LOG.trace(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_race {
    ($jurisdiction:expr, $election:expr, $office:expr) => {
        $crate::util::LOG.race($jurisdiction, $election, $office);
    };
}
