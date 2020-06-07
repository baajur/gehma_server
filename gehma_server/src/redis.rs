use r2d2::Pool;
use r2d2_redis::{r2d2, RedisConnectionManager};

use log::{info, trace};

pub type RedisPool = Pool<RedisConnectionManager>;

/// Connects to redis
#[allow(dead_code)]
pub(crate) fn connect_redis(redis_url: impl Into<String>) -> RedisPool {
    trace!("Entering connect_redis");

    let manager =
        RedisConnectionManager::new(redis_url.into()).expect("Cannot connect to redis");

    info!("Redis was setup");

    let pool = r2d2::Pool::builder().build(manager).unwrap();

    trace!("Exiting connect_redis");

    pool
}
