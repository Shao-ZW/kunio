#[cfg(feature = "debug")]
#[macro_export]
macro_rules! info {
    ($($args:expr),*) => { println!("\x1b[33m{}\x1b[0m", format_args!($($args),*)); }
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! info {
    ($($args:expr),*) => {};
}

pub use info;
