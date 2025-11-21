use std::sync::{Arc, Mutex};

#[macro_export]
macro_rules! string {
    ($x:expr) => {
        String::from($x)
    };
}

#[macro_export]
macro_rules! arc {
    ($x:expr) => {
        Arc::new($x)
    };
}

#[macro_export]
macro_rules! mutex {
    ($x:expr) => {
        Mutex::new($x)
    };
}

#[macro_export]
macro_rules! arc_mutex {
    ($x:expr) => {
        Arc::new(Mutex::new($x))
    }
}