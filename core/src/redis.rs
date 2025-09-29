use std::sync::Arc;
use redis::{Client, Connection};
use tokio::sync::Mutex;

pub struct RedisClient {
    pub connection: Arc<Mutex<Connection>>,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        let connection = client.get_connection()?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

}
