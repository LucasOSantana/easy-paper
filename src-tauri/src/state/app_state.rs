use std::sync::Mutex;
use std::time::Instant;
use crate::models::AppConfig;

pub struct ConfigState(pub Mutex<AppConfig>);

pub struct WindowDisplayState(pub Mutex<Instant>);
