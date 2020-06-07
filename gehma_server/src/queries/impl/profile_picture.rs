use crate::queries::*;
use crate::Pool;
use core::errors::{InternalServerError, ServiceError};
use core::models::dao::*;
use diesel::{prelude::*, PgConnection};
use log::{error, trace};

#[derive(Clone)]
pub struct PgProfilePictureDao {
    pub pool: Pool,
}

impl PersistentProfilePictureDao for PgProfilePictureDao {
    fn get_all(&self, _user: &UserDao) -> Result<Vec<ProfilePictureDao>, ServiceError> {
        trace!("queries/impl/profile_picture/get_all");
        use core::schema::profile_pictures::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let p = profile_pictures
            .load::<ProfilePictureDao>(conn)
            .map_err(|_db_error| {
                error!("{}", _db_error);
                ServiceError::InternalServerError(InternalServerError::DatabaseError(
                    _db_error.to_string(),
                ))
            })?;

        Ok(p)
    }
}
