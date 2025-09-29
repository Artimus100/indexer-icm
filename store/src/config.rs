use std::env;

use dotenvy::dotenv;

pub struct Config {
    pub db_url: String,
}

impl Default for Config {
    fn default() -> Self {
        dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");

        Self {
            db_url: database_url,
        }
    }
}
