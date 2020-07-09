use crate::queries::*;
use crate::Pool;
use chrono::{DateTime, Local};
use core::errors::{InternalServerError, ServiceError};
use core::models::dao::*;
use core::models::dto::*;
use core::models::PhoneNumber;
use diesel::{prelude::*, PgConnection};
use log::{debug, error, info, trace};
use uuid::Uuid;

const INCREASE_XP: i32 = 100;
const BROADCAST_LIMIT: i64 = 20;

#[derive(Clone)]
pub struct PgUserDao {
    pub pool: Pool,
}

impl PersistentUserDao for PgUserDao {
    fn create_analytics_for_user(
        &self,
        user: &UserDao,
    ) -> Result<AnalyticDao, ::core::errors::ServiceError> {
        info!("queries/user/analytics_user");
        use core::schema::analytics::dsl::analytics;

        let ana = AnalyticDao::my_from(user);
        let conn: &PgConnection = &self.pool.get().unwrap();

        diesel::insert_into(analytics)
            .values(&ana)
            .get_result::<AnalyticDao>(conn)
            //.map(|w| w.into())
            .map_err(|_db_error| {
                error!("{}", _db_error);
                ServiceError::BadRequest("Could not log change".into())
            })
    }

    fn create(
        &self,
        tel: &PhoneNumber,
        country_code: &str,
        version: &str,
        access_token: &str,
    ) -> Result<UserDao, ServiceError> {
        info!("create()");
        use core::schema::users::dsl::users;

        let new_inv = UserDao::my_from(&tel.to_string(), country_code, version, access_token);

        let conn: &PgConnection = &self.pool.get().unwrap();

        diesel::insert_into(users)
            .values(&new_inv)
            .get_result::<UserDao>(conn)
            //.map(|w| w.into())
            .map_err(|_db_error| {
                error!("{}", _db_error);
                error!("user {:#?}", new_inv);
                ServiceError::InternalServerError(InternalServerError::DatabaseError(
                    _db_error.to_string(),
                ))
            })
    }

    fn create_usage_statistics_for_user(
        &self,
        user: &UserDao,
    ) -> Result<UsageStatisticEntryDao, ::core::errors::ServiceError> {
        info!("queries/user/analytics_usage_statistics");
        use core::schema::usage_statistics::dsl::usage_statistics;

        let ana = UsageStatisticEntryDao::my_from(user);
        let conn: &PgConnection = &self.pool.get().unwrap();

        diesel::insert_into(usage_statistics)
            .values(&ana)
            .get_result::<UsageStatisticEntryDao>(conn)
            //.map(|w| w.into())
            .map_err(|_db_error| {
                eprintln!("{}", _db_error);
                ServiceError::BadRequest("Could not log change".into())
            })
    }

    fn update_user(
        &self,
        myid: &Uuid,
        user: &UpdateUserDto,
        _current_time: DateTime<Local>,
    ) -> Result<(UserDao, Vec<ContactPushNotificationDao>), ::core::errors::ServiceError> {
        info!("queries/user/update_user_query");
        use core::schema::users::dsl::{
            changed_at, client_version, description, id, led, users, xp,
        };

        //TODO move to controller
        /*
        let xp_limit = self
            .ratelimit_service
            .inner
            .lock()
            .unwrap()
            .check_rate_limit_xp(myid, &self.pool, current_time)?;
        */
        let xp_limit = false; //FIXME remove

        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = users.filter(id.eq(myid));

        let my_led = user.led;

        let inc_xp = match (my_led, xp_limit) {
            (true, false) => INCREASE_XP,
            _ => 0,
        };

        diesel::update(target)
            .set((
                description.eq(user.description.to_string()),
                led.eq(my_led),
                changed_at.eq(chrono::Local::now().naive_local()),
                client_version.eq(user.client_version.clone()),
                xp.eq(xp + inc_xp), // add experience for every event if `my_led` is true
            ))
            .execute(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

        users
            .filter(id.eq(myid))
            .load::<UserDao>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|res_users| {
                Ok(res_users
                    .first()
                    .cloned()
                    .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?)
                //.map(|w| w.into())
            })
            .and_then(|user| {
                if my_led {
                    let contacts = get_users_for_sending_push_notification(&user, &self.pool)?;

                    Ok((user, contacts))
                } else {
                    Ok((user, vec![]))
                }
            })
            .and_then(|(user, contacts)| {
                // Create an broadcast entry
                for contact in contacts.iter() {
                    if let Err(err) = self.create_broadcast_entry(
                        &user,
                        &contact.target_hash_tele_num,
                        &user.description,
                    ) {
                        error!("Cannot make a broadcast entry: {}", err);
                    }
                }

                Ok((user, contacts))
            })
    }

    /// Get the user by uid
    fn get_by_id(&self, myid: &Uuid) -> Result<UserDao, ServiceError> {
        info!("queries/user/get_query");
        use core::schema::users::dsl::{id, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        users
            .filter(id.eq(myid))
            .load::<UserDao>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|w| {
                w.first()
                    .cloned()
                    .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
                //.map(|w| w.into())
            })
    }

    fn get_by_tele_num(&self, phone_number: &PhoneNumber) -> Result<UserDao, ServiceError> {
        info!("queries/user/get_query");
        use core::schema::users::dsl::{tele_num, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        users
            .filter(tele_num.eq(phone_number.to_string()))
            .load::<UserDao>(conn)
            .map_err(|_db_error| {
                error!("db_error: {}", _db_error);
                ServiceError::InternalServerError(InternalServerError::DatabaseError(
                    _db_error.to_string(),
                ))
            })
            .and_then(|w| {
                w.first()
                    .cloned()
                    .ok_or_else(|| ServiceError::ResourceDoesNotExist)
            })
    }

    fn get_by_hash_tele_num_unsafe(
        &self,
        user_hash_tele_num: &HashedTeleNum,
    ) -> Result<UserDao, ServiceError> {
        info!("queries/user/get_query");
        use core::schema::users::dsl::{hash_tele_num, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        users
            .filter(hash_tele_num.eq(user_hash_tele_num))
            .load::<UserDao>(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
            .and_then(|w| {
                w.first()
                    .cloned()
                    .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
                //.map(|w| w.into())
            })
    }

    fn update_profile_picture(
        &self,
        user_id: Uuid,
        user: &UpdateProfilePictureDto,
    ) -> Result<(), ServiceError> {
        trace!("queries/user/update_profile_picture");
        use core::schema::users::dsl::{changed_at, id, profile_picture, users};

        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = users.filter(id.eq(user_id));

        diesel::update(target)
            .set((
                changed_at.eq(chrono::Local::now().naive_local()),
                profile_picture.eq(user.profile_id),
            ))
            .execute(conn)
            .map_err(|_db_error| {
                error!("db_error: {}", _db_error);
                ServiceError::InternalServerError(InternalServerError::DatabaseError(
                    _db_error.to_string(),
                ))
            })?;

        Ok(())
    }

    fn update_token(&self, uid: &Uuid, token: String) -> Result<(), ServiceError> {
        info!("queries/push_notification/update_token_query");
        use core::schema::users::dsl::*;
        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = users.filter(id.eq(uid));

        diesel::update(target)
        .set(
            firebase_token.eq(Some(token))
            //disabled 29.11.2019
            //changed_at.eq(chrono::Local::now().naive_local()),
        )
        .execute(conn)
        .map_err(|_db_error| {
            error!("db_error {}", _db_error);
            ServiceError::BadRequest("Updating state failed".into())
        })?;

        Ok(())
    }

    fn get_profile_picture(&self, user: &UserDao) -> Result<String, ServiceError> {
        let conn: &PgConnection = &self.pool.get().unwrap();

        use diesel::prelude::*;
        use diesel::sql_types::Text;
        use diesel::QueryableByName;
        #[derive(QueryableByName)]
        struct R {
            #[sql_type = "Text"]
            path: String,
        }

        let j = diesel::sql_query(
            "SELECT p.path FROM users JOIN profile_pictures p ON users.profile_picture = p.id WHERE users.id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(user.id)
        .get_result::<R>(conn)
        .map_err(|_db_error| {
            error!("{:?}", _db_error);
            ServiceError::InternalServerError(InternalServerError::DatabaseError(
                _db_error.to_string(),
            ))
        })?;

        Ok(j.path)
    }

    fn get_latest_broadcast(
        &self,
        user: &UserDao,
        mark_seen: bool,
    ) -> Result<Vec<BroadcastElementDao>, ::core::errors::ServiceError> {
        debug!("fn get_latest_broadcast for {} ({})", user.tele_num, user.hash_tele_num);

        use core::schema::broadcast::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let list = broadcast
            .filter(is_seen.eq(false).and(display_user.eq(&user.hash_tele_num)))
            .limit(BROADCAST_LIMIT)
            .order_by(created_at.desc())
            .load::<BroadcastElementDao>(conn)?;

        debug!("BroadcastElements {} found", list.len());

        if mark_seen {
            log::debug!("Update broadcasts as seen");
            self.update_latest_broadcast(user)?;
        }

        Ok(list)
    }

    /// Updates all unseen elements to seen.
    fn update_latest_broadcast(&self, user: &UserDao) -> Result<(), ::core::errors::ServiceError> {
        use core::schema::broadcast::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        let target = broadcast.filter(display_user.eq(&user.hash_tele_num));

        diesel::update(target).set(is_seen.eq(true)).execute(conn)?;

        Ok(())
    }

    /// User `originator_user` creates a broadcast entry for the `disaplay_user`
    fn create_broadcast_entry(
        &self,
        originator_user: &UserDao,
        display_user_hash: &HashedTeleNum,
        mytext: &String,
    ) -> Result<(), ::core::errors::ServiceError> {
        use core::schema::broadcast::dsl::*;

        let conn: &PgConnection = &self.pool.get().unwrap();

        diesel::insert_into(broadcast)
            .values(InsertBroadcastElementDao {
                originator_user_id: originator_user.id,
                display_user: display_user_hash.clone(),
                text: mytext.clone(),
                is_seen: false,
                updated_at: chrono::Local::now().naive_local(),
                created_at: chrono::Local::now().naive_local(),
            })
            .execute(conn)?;

        Ok(())
    }
}

fn get_users_for_sending_push_notification(
    user: &UserDao, //sender
    pool: &Pool,
) -> Result<Vec<ContactPushNotificationDao>, ServiceError> {
    info!("queries/user/get_users_for_sending_push_notification");

    let conn: &PgConnection = &pool.get().unwrap();

    let my_contacts: Vec<ContactPushNotificationDao> = diesel::sql_query(
        "SELECT from_id, name, firebase_token, target_hash_tele_num FROM contact_view WHERE from_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user.id)
    .load::<ContactPushNotificationDao>(conn)
    .map_err(|_db_error| {
        error!("{:?}", _db_error);
        ServiceError::InternalServerError(InternalServerError::DatabaseError(_db_error.to_string()))
    })?
    .into_iter()
    .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS)
    .collect();

    /*
    notify_service.clone().push(
        my_contacts
            .into_iter()
            .map(|c| (c.name, c.firebase_token))
            .collect(),
    )?;*/

    Ok(my_contacts)
}
