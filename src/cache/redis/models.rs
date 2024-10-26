use crate::publish::models::PublishNews;

use redis::{RedisError, RedisResult};
use redis::RedisWrite;
use redis::Value;
use serde::de::Error;

impl redis::ToRedisArgs for PublishNews {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match serde_json::to_string(self) {
            Ok(json_str) => out.write_arg_fmt(json_str),
            Err(err) => {
                tracing::error!("cacher: failed to serialize search parameters: {err:#?}");
            }
        }
    }
}

impl redis::FromRedisValue for PublishNews {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {
            Value::BulkString(data) => {
                serde_json::from_slice::<PublishNews>(data.as_slice()).map_err(RedisError::from)
            }
            _ => {
                let err = serde_json::Error::custom("failed to extract redis value type");
                Err(RedisError::from(err))
            }
        }
    }
}
