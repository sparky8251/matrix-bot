use std::env;
use std::fmt;
use tracing_subscriber::FmtSubscriber;

enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

enum Filter {
    MatrixBot,
    All,
}

impl Default for Level {
    fn default() -> Self {
        Self::Info
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::MatrixBot
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MatrixBot => write!(f, "matrix_bot="),
            Self::All => write!(f, ""),
        }
    }
}

impl From<String> for Level {
    fn from(string: String) -> Self {
        match string.as_str() {
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => Self::default(),
        }
    }
}

impl From<String> for Filter {
    fn from(string: String) -> Self {
        match string.as_str() {
            "matrix_bot" => Self::MatrixBot,
            "matrix-bot" => Self::MatrixBot,
            _ => Self::default(),
        }
    }
}

fn log_filter(level: Level, filter: Filter) -> String {
    String::from(filter.to_string() + &level.to_string())
}

pub fn init() {
    let level = match env::var("MATRIX_BOT_LOG_LEVEL") {
        Ok(v) => Level::from(v),
        Err(_) => Level::default(),
    };
    let filter = match env::var("MATRIX_BOT_LOG_FILTER") {
        Ok(v) => Filter::from(v),
        Err(_) => Filter::default(),
    };
    FmtSubscriber::builder()
        .with_env_filter(log_filter(level, filter))
        .init();
}
