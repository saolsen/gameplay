use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref SENTRY_DSN: String = env::var("SENTRY_DSN").unwrap();
    pub static ref SENTRY_ENV: String = env::var("SENTRY_ENV").unwrap();
    pub static ref CLERK_PUB_API_KEY: String = env::var("CLERK_PUB_API_KEY").unwrap();
    pub static ref CLERK_PUB_ENCRYPTION_KEY: String = env::var("CLERK_PUB_ENCRYPTION_KEY").unwrap();
    pub static ref DB: String = env::var("DB").unwrap();
}

pub fn load() {
    let _ = *SENTRY_DSN;
    let _ = *SENTRY_ENV;
    let _ = *CLERK_PUB_API_KEY;
    let _ = *CLERK_PUB_ENCRYPTION_KEY;
    let _ = *DB;
}
