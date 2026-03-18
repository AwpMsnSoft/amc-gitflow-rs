use colored::{Color, Colorize};
use std::fmt::Display;

pub(crate) enum Level {
    Info,
    Success,
    Warn,
    Error,
    Debug,
    Item,
}

pub(crate) fn colored_display<S: Display>(msg: S, level: Level) {
    let (prefix, color) = match level {
        Level::Info => ("➥", Color::BrightWhite),
        Level::Success => ("✔", Color::BrightGreen),
        Level::Warn => ("⚠", Color::BrightYellow),
        Level::Error => ("✗", Color::BrightRed),
        Level::Debug => ("♨", Color::BrightBlack),
        Level::Item => ("•", Color::White),
    };
    let message = format!("{}  {}", prefix, msg).color(color);
    match level {
        Level::Info | Level::Success | Level::Debug | Level::Item => println!("{}", message),
        Level::Warn | Level::Error => eprintln!("{}", message),
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Info)
    };
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Success)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Warn)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Error)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) || std::env::var("AMC_GITFLOW_DEBUG").is_ok() {
            $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Debug)
        }
    };
}

#[macro_export]
macro_rules! item {
    ($($arg:tt)*) => {
        $crate::utils::output::colored_display(format!($($arg)*), $crate::utils::output::Level::Item)
    };
}
