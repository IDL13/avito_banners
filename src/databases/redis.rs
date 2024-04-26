use jwt::SignWithKey;
use redis::{Commands, Connection, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value};
use std::fmt::Error;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::collections::BTreeMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const SECCESS: &'static str = "[SECCESS OPERATION]";

pub struct Redis {
    pub conn: Connection,
}

impl Redis {
    pub fn new() -> Self {
        Self {
           conn: connection_redis(),
        }
    }

    pub fn set(&mut self, tag_id: i32, feature_id: i32, val: Vec<String>) -> &str {

        let hash = hash_str(tag_id, feature_id);
        
        let result: Result<(), RedisError> = self.conn.hset_multiple(hash, &[("title", val[0].clone()), ("text", val[1].clone()), ("url", val[2].clone())]);
        match result {
            Ok(_) => return SECCESS,
            Err(err) => {
                println!("Error: {}", err);
                panic!("{}", err)
            }
        }
    }

    pub fn get(&mut self, tag_id: i32, feature_id: i32) -> Result<(String, String, String), RedisError> {
        let key = hash_str(tag_id, feature_id);
        let title: String = self.conn.hget(key.clone(), "title")?;
        let text: String = self.conn.hget(key.clone(), "text")?;
        let url: String = self.conn.hget(key, "url")?;

        Ok((title, text, url))
    }

    pub fn del(&mut self, key: String) -> Result<(), RedisError> {
        self.conn.del(key)     
    }

    pub fn key_count(&mut self) -> Result<usize, RedisError> {
        let keys: Vec<HashMap<String, i32>> = self.conn.keys("*")?;
        Ok(keys.len())
    }

    pub fn check(&mut self, tag_id: i32, feature_id: i32) -> bool {
        let hash = hash_str(tag_id, feature_id);

        match self.get(tag_id, feature_id) {
            Ok(_) => {
                return true
            },
            Err(err) => {
                println!("{}", err);
                return false
            }
        }
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

fn hash_str(tag_id: i32, feature_id: i32) -> String {
    let hash_key: Vec<u8> = [tag_id.to_le_bytes(), feature_id.to_le_bytes()].concat();

    let key: Hmac<Sha256> = Hmac::new_from_slice(&hash_key).expect("Error from conversion");
    let mut claims = BTreeMap::new();
    claims.insert("sub", "someone");
    claims.sign_with_key(&key).expect("Error from caching")
}