use crate::Pool;
use core::errors::ServiceError;
use core::models::PhoneNumber;
use core::models::dao::*;
use core::models::dto::*;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;
use crate::ratelimits::RateLimitWrapper;
use web_contrib::push_notifications::NotifyService;
use chrono::{DateTime, Local};
use crate::persistence::user::PersistentUserDao;

//use crate::routes::user::{ResponseContact, UpdateUser};

use log::{error, info};

const INCREASE_XP : i32 = 100;
const PROFILE_WIDTH: u32 = 500;
const PROFILE_HEIGHT: u32 = 500;

#[derive(Clone)]
pub struct PgUserDao {
    pub pool: Pool,
    pub notify_service: NotifyService,
    pub ratelimit_service: RateLimitWrapper,
}

impl PersistentUserDao for PgUserDao {

    /*
fn get_by_tele_num(
    &self,
    tele: &PhoneNumber,
) -> Result<UserDto, ::core::errors::ServiceError> {
    info!("queries/user/get_entry_by_tel_query");

    use core::schema::users::dsl::*;

    let conn: &PgConnection = &self.pool.get().unwrap();

    let tele = tele.to_string();

    let res = users
        .filter(tele_num.eq(tele))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })?;

    res.first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
        .map(|w| w.into())
}
    */

fn get_contacts(
    &self,
    user: &UserDto,
) -> Result<Vec<ContactDto>, ::core::errors::ServiceError> {
    info!("queries/user/get_contacts");

    use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
    use core::schema::contacts::dsl::{contacts, from_id, name, target_hash_tele_num};
    use core::schema::users::dsl::*;

    let conn: &PgConnection = &self.pool.get().unwrap();

    contacts
        .filter(from_id.eq(user.id))
        .inner_join(users.on(hash_tele_num.eq(target_hash_tele_num)))
        .left_join(
            blacklist.on(
                target_hash_tele_num
                .eq(hash_blocked)
                .and(hash_blocker.eq(&user.hash_tele_num)))
                /*hash_tele_num
                    .eq(hash_blocker)
                    .and(hash_blocked.eq(&user.hash_tele_num)))*/,
        )
        .select((
            id,
            name,
            tele_num,
            led,
            country_code,
            description,
            changed_at,
            profile_picture,
            hash_tele_num,
            hash_blocked.nullable(),
            xp, 
            created_at,
            client_version,
            firebase_token.nullable(),
            access_token,
        ))
        .distinct()
        .load::<(
            Uuid, //id
            String, //name
            String, //tele_num
            bool, //led
            String, //cc
            String, //description
            chrono::NaiveDateTime,
            String, //hash_tele
            String, //hash_blocked
            Option<String>, 
            i32, //XP
            chrono::NaiveDateTime,//created_at
            String, //client
            Option<String>, //firebase
            String, //access_token
        )>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|values| {
            Ok(values
                .into_iter()
                .map(
                    |(
                        _id,
                        _name,
                        _tele_num,
                        _led,
                        _country_code,
                        _description,
                        _changed_at,
                        _profile_picture,
                        _hash_tele_num,
                        _blocked,
                        _xp,
                        _created_at,
                        _client_version,
                        _firebase_token,
                        _access_token,
                    )| {
                        let user_d = UserDao {
                            id: _id,
                            tele_num: _tele_num,
                            led: _led,
                            created_at: _created_at,
                            country_code: _country_code,
                            description: _description,
                            changed_at: _changed_at,
                            client_version: _client_version,
                            firebase_token: _firebase_token,
                            profile_picture: _profile_picture,
                            access_token: _access_token,
                            hash_tele_num: _hash_tele_num,
                            xp: _xp,
                        };

                        ContactDto::new(
                            _name,
                            _blocked.is_some(),
                            user_d.into()
                        )
                    },
                )
                .collect())
        })
}

fn create_analytics_for_user(
    &self,
    user: &UserDto,
) -> Result<AnalyticDto, ::core::errors::ServiceError> {
    info!("queries/user/analytics_user");
    use core::schema::analytics::dsl::analytics;

    let ana = AnalyticDao::my_from(user);
    let conn: &PgConnection = &self.pool.get().unwrap();

    diesel::insert_into(analytics)
        .values(&ana)
        .get_result::<AnalyticDao>(conn)
        .map(|w| w.into())
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })
}

fn create(
    &self,
    tel: &PhoneNumber,
    country_code: &str,
    version: &str,
    access_token: &str,
) -> Result<UserDto, ServiceError> {
    info!("queries/user/create_query");
    use core::schema::users::dsl::users;

    let new_inv = UserDao::my_from(&tel.to_string(), country_code, version, access_token);
    let conn: &PgConnection = &self.pool.get().unwrap();

    diesel::insert_into(users)
        .values(&new_inv)
        .get_result::<UserDao>(conn)
        .map(|w| w.into())
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Cannot insert user".into())
        })

}

fn create_usage_statistics_for_user(
    &self,
    user: &UserDto,
) -> Result<UsageStatisticEntryDto, ::core::errors::ServiceError> {
    info!("queries/user/analytics_usage_statistics");
    use core::schema::usage_statistics::dsl::usage_statistics;

    let ana = UsageStatisticEntryDao::my_from(user);
    let conn: &PgConnection = &self.pool.get().unwrap();

    diesel::insert_into(usage_statistics)
        .values(&ana)
        .get_result::<UsageStatisticEntryDao>(conn)
        .map(|w| w.into())
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })
}

fn update_user(
    &self,
    myid: &Uuid,
    user: &UpdateUserDto,
    current_time: DateTime<Local>,
) -> Result<UserDto, ::core::errors::ServiceError> {
    info!("queries/user/update_user_query");
    use core::schema::users::dsl::{changed_at, client_version, description, id, led, users, xp};

    let xp_limit = self.ratelimit_service.inner.lock().unwrap().check_rate_limit_xp(myid, &self.pool, current_time)?;

    let conn: &PgConnection = &self.pool.get().unwrap();

    let target = users.filter(id.eq(myid));

    let my_led = user.led;

    let inc_xp = match (my_led, xp_limit) {
        (true, false) => INCREASE_XP,
        _ => 0
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
                .map(|w| w.into())
        })
        .and_then(|user| {
            if my_led {
                //Sending push notification
                if !self.ratelimit_service.inner.lock().unwrap().check_rate_limit_updates(&myid, &self.pool, current_time)? {
                    sending_push_notifications(&user, &self.pool, &self.notify_service).map_err(|err| {
                        error!("{}", err);
                        ServiceError::BadRequest("Cannot send push notifications".to_string())
                    })?;

                    //return Err(ServiceError::RateLimit("No push notification sent. Try again later".to_string())); 
                }
                else {
                    info!("Ratelimit reached, not sending push notification");
                }

            }

            Ok(user)
        })
}

/// Get the user by uid
fn get_by_id(
    &self,
    myid: &Uuid,
    my_access_token: String,
) -> Result<UserDto, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{access_token, id, users};

    let conn: &PgConnection = &self.pool.get().unwrap();

    users
        .filter(id.eq(myid).and(access_token.eq(my_access_token)))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
                .map(|w| w.into())
        })
}

fn get_by_tele_num(
    &self,
    phone_number: &PhoneNumber,
    my_access_token: String,
) -> Result<UserDto, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{access_token, tele_num, users};

    let conn: &PgConnection = &self.pool.get().unwrap();

    users
        .filter(
            tele_num
                .eq(phone_number.to_string())
                .and(access_token.eq(my_access_token)),
        )
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
                .map(|w| w.into())
        })
}

fn update_profile_picture(
    &self,
    user: &UserDto,
) -> Result<(), ServiceError> {
    info!("queries/user/update_profile_picture");
    use core::errors::InternalError;
    use core::schema::users::dsl::{id, profile_picture, users};

    let conn: &PgConnection = &self.pool.get().unwrap();

    let target = users.filter(id.eq(user.id));

    //FIXME add better error message
    let path = format!("static/profile_pictures/{}.jpg", user.hash_tele_num);
    let _ = img_profile::generate(PROFILE_HEIGHT, PROFILE_WIDTH, path.clone())
        .map_err(InternalError::GenerateImage)?;

    diesel::update(target)
        .set((
            //changed_at.eq(chrono::Local::now().naive_local()),
            profile_picture.eq(&path),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    info!("Updating profile {}", path);

    Ok(())
}

fn update_token(
    &self,
    uid: &Uuid,
    token: String,
) -> Result<(), ServiceError> {
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
}

fn sending_push_notifications(
    user: &UserDto, //this is the sender
    pool: &Pool,
    notify_service: &NotifyService,
) -> Result<(), ServiceError> {
    info!("queries/user/sending_push_notifications");
    use diesel::sql_types::{Text, Uuid};
    use diesel::{Queryable, QueryableByName};

    let conn: &PgConnection = &pool.get().unwrap();

    type FromId = uuid::Uuid;
    type Name = String;
    type FirebaseToken = String;

    #[derive(Debug, Deserialize, Clone, Queryable, QueryableByName)]
    struct DatabaseResponse {
        #[sql_type = "Uuid"]
        from_id: FromId,
        #[sql_type = "Text"]
        name: Name,
        #[sql_type = "Text"]
        firebase_token: FirebaseToken,
    };

    let my_contacts: Vec<DatabaseResponse> = diesel::sql_query(
        "SELECT from_id, name, firebase_token FROM contact_view WHERE from_id = $1",
    )
    .bind::<Uuid, _>(user.id)
    .load::<DatabaseResponse>(conn)
    .map_err(|_db_error| {
        error!("{:?}", _db_error);
        ServiceError::BadRequest("Database error".into())
    })?
    .into_iter()
    .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS)
    .collect();

    //FIXME check
    notify_service.clone().service.lock().unwrap().push(
        my_contacts
            .into_iter()
            .map(|c| (c.name, c.firebase_token))
            .collect(),
    )?;

    Ok(())
}
