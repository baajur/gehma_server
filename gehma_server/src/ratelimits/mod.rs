use crate::Pool;
use chrono::prelude::*;
use chrono::{Duration, Local};
use core::errors::{ServiceError, InternalServerError};
use core::models::dao::*;
use diesel::{prelude::*, PgConnection};
use log::{debug, error, info};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

pub type MyDateTime = DateTime<Local>;

#[derive(Clone)]
pub struct RateLimitWrapper {
    pub inner: Arc<Mutex<Box<dyn RateLimitPolicy + 'static + Send>>>,
}

impl RateLimitWrapper {
    pub fn new(a: Box<dyn RateLimitPolicy + 'static + Send>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(a)),
        }
    }
}

pub trait RateLimitPolicy {
    /// Returns true if limit was reached
    fn check_rate_limit_updates(
        &self,
        myid: &Uuid,
        pool: &Pool,
        current_time: MyDateTime,
    ) -> Result<bool, ServiceError>;

    /// Returns true if limit was reached
    fn check_rate_limit_xp(
        &self,
        myid: &Uuid,
        pool: &Pool,
        current_time: MyDateTime,
    ) -> Result<bool, ServiceError>;
}

pub struct DefaultRateLimitPolicy;

impl DefaultRateLimitPolicy {
    fn check_rate_limit(
        &self,
        myid: &Uuid,
        pool: &Pool,
        current_time: MyDateTime,
        limit: usize,
    ) -> Result<bool, ServiceError> {
        info!("ratelimits/mod/check_rate_limit");

        use core::schema::analytics::dsl::{analytics, created_at, tele_num};
        use core::schema::users::dsl::{id, users};

        let conn: &PgConnection = &pool.get().unwrap();

        users
            .filter(id.eq(myid))
            .load::<UserDao>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|res_users| {
                Ok(res_users
                    .first()
                    .cloned()
                    .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?)
            })
            .and_then(|user| {
                debug!("User found");

                //TODO extract
                let duration = Duration::minutes(10);

                let threshold = (current_time - duration).naive_local();

                debug!("Threshold is {}", threshold);

                let count = analytics
                    .filter(tele_num.eq(user.tele_num).and(created_at.ge(threshold)))
                    .load::<AnalyticDao>(conn)
                    .map_err(|_db_error| {
                        error!("db error: {}", _db_error);
                        ServiceError::InternalServerError(InternalServerError::DatabaseError(_db_error.to_string()))
                    })?
                    .len();

                Ok(count > limit)
            })
    }
}

impl RateLimitPolicy for DefaultRateLimitPolicy {
    fn check_rate_limit_updates(
        &self,
        myid: &Uuid,
        pool: &Pool,
        current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        //TODO move into configuration
        const LIMIT: usize = 3;

        self.check_rate_limit(myid, pool, current_time, LIMIT)
    }

    fn check_rate_limit_xp(
        &self,
        myid: &Uuid,
        pool: &Pool,
        current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        //TODO move into configuration
        const LIMIT: usize = 2;

        self.check_rate_limit(myid, pool, current_time, LIMIT)
    }
}

#[allow(dead_code)]
pub struct TestingTrueRateLimitPolicy;

impl RateLimitPolicy for TestingTrueRateLimitPolicy {
    fn check_rate_limit_updates(
        &self,
        _myid: &Uuid,
        _pool: &Pool,
        _current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        Ok(true)
    }

    fn check_rate_limit_xp(
        &self,
        _myid: &Uuid,
        _pool: &Pool,
        _current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        Ok(true)
    }
}

#[allow(dead_code)]
pub struct TestingFalseRateLimitPolicy;

impl RateLimitPolicy for TestingFalseRateLimitPolicy {
    fn check_rate_limit_updates(
        &self,
        _myid: &Uuid,
        _pool: &Pool,
        _current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        Ok(false)
    }

    fn check_rate_limit_xp(
        &self,
        _myid: &Uuid,
        _pool: &Pool,
        _current_time: MyDateTime,
    ) -> Result<bool, ServiceError> {
        Ok(false)
    }
}
