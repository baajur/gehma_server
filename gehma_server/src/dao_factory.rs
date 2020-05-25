use crate::{Pool, RedisPool};

use crate::queries::*;

pub struct DaoFactory(Pool, RedisPool);

impl DaoFactory {
    pub fn new(pool: Pool, pool_redis: RedisPool) -> Self {
        Self(pool, pool_redis)
    }

    pub fn get_user_dao(&self) -> Box<dyn PersistentUserDao> {
        Box::new(PgUserDao {
            pool: self.0.clone(),
        })
    }

    pub fn get_blacklist_dao(&self) -> Box<dyn PersistentBlacklistDao> {
        Box::new(PgBlacklistDao {
            pool: self.0.clone(),
        })
    }

    pub fn get_contacts_dao(&self) -> Box<dyn PersistentContactsDao> {
        Box::new(PgContactsDao {
            pool: self.0.clone(),
        })
    }

    pub fn get_session_dao(&self) -> Box<dyn PersistentSessionDao> {
        Box::new(RedisSessionDao {
            redis_pool: self.1.clone(),
        })
    }
}
