use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::dao::*;
use core::models::dto::*;
use core::models::PhoneNumber;

use crate::Pool;

use crate::queries::*;
use log::{error, info};

#[derive(Clone)]
pub struct PgBlacklistDao {
    pub pool: Pool,
}

impl PersistentBlacklistDao for PgBlacklistDao {
    fn get(&self, sblocker: Uuid) -> Result<Vec<BlacklistDto>, ServiceError> {
        info!("queries/blacklist/get_query");
        use core::schema::blacklist::dsl::{blacklist, hash_blocker};
        use core::schema::users::dsl::{id, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        let user = users
            .filter(id.eq(sblocker))
            .load::<UserDao>(conn)
            .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))?
            .first()
            .cloned()
            .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?;

        blacklist
            .filter(hash_blocker.eq(user.hash_tele_num))
            .load::<BlacklistDao>(conn)
            .map(|w| w.into_iter().map(|k| k.into()).collect())
            .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))
    }

    fn create(
        &self,
        blocker: &PhoneNumber,
        blocked: &PhoneNumber,
    ) -> Result<BlacklistDto, ServiceError> {
        info!("queries/blacklist/create_query");
        use core::schema::blacklist::dsl::blacklist;

        let conn: &PgConnection = &self.pool.get().unwrap();
        let new_inv: BlacklistDao = BlacklistDao::my_from(blocker, blocked);

        //println!("{:?}", new_inv);

        diesel::insert_into(blacklist)
            .values(&new_inv)
            .get_result::<BlacklistDao>(conn)
            .map(|w| w.into())
            .map_err(|_db_error| {
                error!("{:?}", _db_error);
                ServiceError::BadRequest("Cannot insert into blacklist".into())
            })
    }

    fn delete(&self, sblocker: &HashedTeleNum, sblocked: &HashedTeleNum) -> Result<(), ServiceError> {
        info!("queries/blacklist/delete_query");
        use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};

        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = blacklist
            .filter(hash_blocker.eq(sblocker))
            .filter(hash_blocked.eq(sblocked));

        diesel::delete(target)
            .execute(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Cannot delete blacklist".into()))?;

        Ok(())
    }
}
