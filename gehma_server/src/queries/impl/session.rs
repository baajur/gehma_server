use crate::queries::*;
use crate::RedisPool;
use r2d2_redis::redis::{cmd};
use uuid::Uuid;
use core::errors::ServiceError;

#[derive(Clone)]
pub struct RedisSessionDao {
    pub redis_pool: RedisPool,
}

pub fn get_default_expire() -> i64 {
    28800 //8 hours
}

impl PersistentSessionDao for RedisSessionDao {
    fn set_new_session(
        &self,
        id: &Uuid,
        session: &str,
        expire: Option<i64>,
    ) -> Result<(), ServiceError> {
        let mut connection = self.redis_pool.get().unwrap();

        cmd("SET")
            .arg(id.to_string())
            .arg(session)
            .execute(&mut *connection);

        cmd("EXPIRE")
            .arg(id.to_string())
            .arg(expire.unwrap_or(get_default_expire()))
            .execute(&mut *connection);

        Ok(())
    }

    fn clear_session(&self, id: &Uuid) -> Result<(), ServiceError> {
        let mut connection = self.redis_pool.get().unwrap();

        cmd("DEL")
            .arg(id.to_string())
            .execute(&mut *connection);

        Ok(())
    }
}
