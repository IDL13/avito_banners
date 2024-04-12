use redis::{Commands, Connection, FromRedisValue, RedisError, RedisResult, Value};
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const SECCESS: &'static str = "[SECCESS OPERATION]";

pub struct Redis {
    conn: Connection,
}

#[derive(Serialize, Deserialize)]
struct Wallet {
    wallet: String,
}

impl Redis {
    pub fn new() -> Self {
        Self {
           conn: connection_redis(),
        }
    }

    pub fn set(&mut self, key: HashMap<String, i32>, value: (String, String, String)) -> &str {
        let _: () = self.conn.set(key, value).unwrap();
        SECCESS
    }

    pub fn get(&mut self, key: HashMap<String, i32>) -> redis::RedisResult<(String, String, String)> {
        self.conn.hgetall(key)
    }

    pub fn del(&mut self, key: HashMap<String, i32>) -> Result<(), RedisError> {
        self.conn.del(key)     
    }

    pub fn key_count(&mut self) -> Result<usize, RedisError> {
        let keys: Vec<HashMap<String, i32>> = self.conn.keys("*")?;
        Ok(keys.len())
    }
}

fn connection_redis() -> Connection {
    match redis::Client::open("redis://127.0.0.1/") {
    // match redis::Client::open("redis://redis/") { 
        Ok(client) => match client.get_connection() {
            Ok(conn) => return conn,
            Err(e) => panic!("Bad redis connection {}", e),
        },
        Err(e) => panic!("Bad redis connection {}", e),
    }
}