use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref SENTRY_DSN: String = env::var("SENTRY_DSN").unwrap();
    pub static ref SENTRY_ENV: String = env::var("SENTRY_ENV").unwrap();
    pub static ref CLERK_PUB_API_KEY: String = env::var("CLERK_PUB_API_KEY").unwrap();
    pub static ref CLERK_PUB_ENCRYPTION_KEY: String = env::var("CLERK_PUB_ENCRYPTION_KEY").unwrap();
    pub static ref DB: String = env::var("DB").unwrap();
    pub static ref QSTASH_URL: String = env::var("QSTASH_URL").unwrap();
    pub static ref QSTASH_TOKEN: String = env::var("QSTASH_TOKEN").unwrap();
    pub static ref QSTASH_CURRENT_SIGNING_KEY: String =
        env::var("QSTASH_CURRENT_SIGNING_KEY").unwrap();
    pub static ref QSTASH_NEXT_SIGNING_KEY: String = env::var("QSTASH_NEXT_SIGNING_KEY").unwrap();
    pub static ref ROOT_URL: String = env::var("ROOT_URL").unwrap();
}

pub fn load() {
    let _ = *SENTRY_DSN;
    let _ = *SENTRY_ENV;
    let _ = *CLERK_PUB_API_KEY;
    let _ = *CLERK_PUB_ENCRYPTION_KEY;
    let _ = *DB;
    let _ = *QSTASH_URL;
    let _ = *QSTASH_TOKEN;
    let _ = *QSTASH_CURRENT_SIGNING_KEY;
    let _ = *QSTASH_NEXT_SIGNING_KEY;
    let _ = *ROOT_URL;
}
