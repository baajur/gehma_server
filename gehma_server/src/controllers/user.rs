use actix_web::{web, HttpRequest};
use chrono::{DateTime, Local};
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;

use crate::queries::*;
use crate::services::push_notifications::NotificationService;
use crate::services::session::*;
use log::{debug, error, info, trace};

//use crate::routes::user::{ResponseContact, UpdateTokenPayload, UpdateUser};
use crate::get_user_by_id;
use crate::routes::user::UpdateTokenPayload;

pub(crate) fn user_signin(
    request: HttpRequest,
    body: PostUserDto,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    current_time: DateTime<Local>,
    notification_service: web::Data<NotificationService>,
    session_service: web::Data<SessionService>,
) -> Result<UserDto, ServiceError> {
    info!("controllers/user/user_signin");

    let access_token = request
        .headers()
        .get("ACCESS_TOKEN")
        .ok_or(ServiceError::BadRequest(
            "Missing access token in ACCESS_TOKEN header".to_string(),
        ))?;

    if !crate::ALLOWED_CLIENT_VERSIONS.contains(&body.client_version.as_str()) {
        error!("Version mismatch. Server does not suppoert client version");
        return Err(ServiceError::BadRequest(format!(
            "Version mismatch. The supported versions are {:?}",
            crate::ALLOWED_CLIENT_VERSIONS
        )));
    }

    let country_code = &body.country_code;
    let tele = PhoneNumber::my_from(&body.tele_num, country_code)?;

    let user = user_dao.get_ref().get_by_tele_num(&tele)?;

    // Check if authorized
    if &user.access_token != access_token {
        return Err(ServiceError::Unauthorized);
    }

    // Update client version
    if user.client_version != body.client_version {
        update_user(
            &user.id.to_string(),
            &UpdateUserDto {
                description: user.description.clone(),
                led: user.led,
                client_version: body.client_version.clone(),
            },
            &user_dao,
            current_time,
            notification_service,
        )?;
    }

    // Generate a new profile picture on every signin
    //user_dao.get_ref().update_profile_picture(&user)?;

    user_dao.get_ref().create_usage_statistics_for_user(&user)?;

    // Set a new session token
    let (session_token, _) = session_service.new_session(user.id);

    let path = user_dao.get_profile_picture(&user)?;

    let mut dto: UserDto = user.into(path);

    dto.session_token = Some(session_token);

    Ok(dto)
}

pub(crate) fn get_entry(
    uid: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
) -> Result<UserDto, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user = get_user_by_id!(user_dao, &parsed);
    let user = user?;

    let path = user_dao.into_inner().get_profile_picture(&user)?;

    let mut user: UserDto = user.into(path);

    // Do not display access token on normal GET
    // It will be only sent on `request_code`
    user.access_token = None;

    Ok(user)
}

pub(crate) fn update_token_handler(
    uid: String,
    payload: UpdateTokenPayload,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;
    let user = get_user_by_id!(user_dao, &parsed);

    user?;

    user_dao.get_ref().update_token(&parsed, payload.token)
}

pub(crate) fn update_user(
    uid: &String,
    update_user: &UpdateUserDto,
    user_dao: &web::Data<Box<dyn PersistentUserDao>>,
    current_time: DateTime<Local>,
    notification_service: web::Data<NotificationService>,
) -> Result<UserDto, ::core::errors::ServiceError> {
    info!("controllers/user/update_user_without_auth");
    let parsed = Uuid::parse_str(uid)?;
    let _user = get_user_by_id!(user_dao, &parsed);

    _user?;

    let (user, contacts) = user_dao
        .get_ref()
        .update_user(&parsed, update_user, current_time)?;

    debug!("Contacts sending push_notifications {}", contacts.len());

    let contacts = contacts
        .into_iter()
        .filter_map(|c| {
            if c.firebase_token.is_some() && c.firebase_token != Some("".to_string()) {
                Some((c.name, c.firebase_token.unwrap()))
            } else {
                debug!("Filtering contact {} because no token", c.name);
                None
            }
        })
        .collect();


    if update_user.led && user.led {
        info!("Contacts {:?}", contacts);
        // Sending push notification
        notification_service.into_inner().push(contacts)?;
    }

    // Log the user update change
    user_dao.get_ref().create_analytics_for_user(&user)?;

    let path = user_dao.get_profile_picture(&user)?;

    Ok(user.into(path))
}

pub(crate) fn change_profile_picture(
    uid: &String,
    user_dao: &web::Data<Box<dyn PersistentUserDao>>,
    update: &UpdateProfilePictureDto,
) -> Result<(), ServiceError> {
    trace!("controllers/user/update_user_without_auth");

    let parsed = Uuid::parse_str(uid)?;
    let _user = get_user_by_id!(user_dao, &parsed);

    user_dao.update_profile_picture(_user?.id, update)?;

    Ok(())
}

/*
pub(crate) fn save_file(
    uid: String,
    field: Field,
    pool: web::Data<Pool>,
    access_token: &String,
    _auth: web::Data<Auth>,
) -> impl Future<Item = i64, Error = ServiceError> {
    use std::fs::OpenOptions;

    info!("controllers/user/save_file");

    let str_content_length = field.headers().get("Content-Length");

    if let Some(str_content_length) = str_content_length {
        let content_length = match str_content_length.to_str().unwrap().parse::<usize>() {
            Ok(le) => le,
            Err(e) => {
                error!("Invalid content length {}", e);
                return Either::A(err(ServiceError::InternalServerError));
            }
        };

        if content_length / 1000 > crate::ALLOWED_PROFILE_PICTURE_SIZE {
            return Either::A(err(ServiceError::BadRequest(
                "Picture is too big".to_string(),
            )));
        }
    } else {
        error!("No content length");
        return Either::A(err(ServiceError::InternalServerError));
    }

    let unsanitized_ending = match parse_content_disposition_to_fileending(
        field
            .headers()
            .get("content-disposition")
            .map(|w| w.to_str().unwrap()),
    ) {
        Ok(end) => end,
        Err(e) => {
            error!("Cannot parse file ending");
            error!("{:?}", e);
            return Either::A(err(ServiceError::InternalServerError));
        }
    };

    //debug!("unsanitized_ending {:?}", unsanitized_ending);

    let ending = match &*unsanitized_ending {
        "jpg" => "jpg",
        "jpeg" => "jpg",
        "png" => "png",
        end => {
            error!("Cannot get file ending {}", end);
            return Either::A(err(ServiceError::InternalServerError));
        }
    }
    .to_string();

    //debug!("ending {:?}", ending);

    let parsed = match Uuid::parse_str(&uid) {
        Ok(p) => p,
        Err(_e) => {
            error!("uuid is invalid {}", uid);
            return Either::A(err(ServiceError::InternalServerError));
        }
    };

    //Authentication
    //Cannot use the `authenticate_user_by_uid!` macro, because
    //return types don't match
    let user = crate::queries::user::get_query(parsed, &access_token, &pool);

    if let Err(my_err) = user {
        error!("{:?}", my_err);
        return Either::A(err(ServiceError::Unauthorized));
    }

    //Authentication END

    let file = match OpenOptions::new()
        .write(true)
        .create(true)
        .open(&format!("static/profile_pictures/{}.{}", uid, ending))
    {
        Ok(file) => file,
        Err(e) => {
            error!("save_file {}", e);
            return Either::A(err(ServiceError::BadRequest("Uuid is invalid".to_string())));
        }
    };

    let ending2 = ending.clone();
    let pool2 = pool.clone();

    Either::B(
        field
            .fold((file, 0i64), move |(mut file, mut acc), bytes| {
                web::block(move || {
                    file.write_all(bytes.as_ref()).map_err(|e| {
                        error!("file.write_all failed {:?}", e);
                        MultipartError::Payload(PayloadError::Io(e))
                    })?;
                    acc += bytes.len() as i64;
                    Ok((file, acc))
                })
                .map_err(|e: BlockingError<MultipartError>| match e {
                    BlockingError::Error(e) => e,
                    BlockingError::Canceled => MultipartError::Incomplete,
                })
            })
            .map(|(_, acc)| acc)
            .map_err(|e| {
                error!("save_file failed, {:?}", e);
                ServiceError::InternalServerError
            })
            .and_then(move |w| {
                remove_old_profile_picture(parsed, ending, pool);

                Ok(w)
            })
            .and_then(move |w| {
                crate::queries::user::update_profile_picture(parsed, ending2, pool2).map_err(
                    |e| {
                        error!("update_profile_picture failed {:?}", e);
                        ServiceError::InternalServerError
                    },
                )?;

                Ok(w)
            }),
    )
}
*/

/*
///The user can upload multiple types of images. When he uploads an image with
///a different type, it may happen that the old won't be overwritten.
fn remove_old_profile_picture(myid: Uuid, ending: String, pool: web::Data<Pool>) {
    info!("controllers/user/remove_old_profile_picture");

    use core::schema::users::dsl::users;

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .find(myid)
        .first::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            use std::fs::remove_file;

            let pp = result.profile_picture.clone();
            let splitted = pp.split('.').collect::<Vec<_>>();

            if let Some(db_end) = splitted.get(1) {
                // If database's profile picture ending is the same as the uploaded one
                // we need not to delete it, because it gets overwritten
                if **db_end == ending {
                    return Ok(());
                }
            }

            if let Err(e) = remove_file(result.profile_picture.clone()) {
                error!("Cannot remove profile picture {}", result.profile_picture);
                error!("Thrown {}", e);
            }

            Ok(())
        })
        .unwrap();
}

fn parse_content_disposition_to_fileending(raw: Option<&str>) -> Result<String, ServiceError> {
    match raw {
        Some(s) => {
            let splitted = s.split(';').collect::<Vec<_>>();
            //form-data; name=\"image\"; filename=\"IMG-20191019-WA0023.jpg\"

            if let Some(f) = splitted.get(2) {
                let ssplitted = f.split('=').collect::<Vec<_>>();

                if let Some(filename) = ssplitted.get(1) {
                    let parsed_filename = filename.trim().replace("\"", "");

                    Ok(parsed_filename
                        .split('.')
                        .collect::<Vec<_>>()
                        .get(1)
                        .unwrap()
                        .to_string())
                } else {
                    error!("No filename in form-data");
                    //FIXME change to BadRequest
                    Err(ServiceError::InternalServerError)
                }
            } else {
                error!("No filename in form-data");
                //FIXME change to BadRequest
                return Err(ServiceError::InternalServerError);
            }
        }

        None => {
            error!("No content-disposition in form-data");
            //FIXME change to BadRequest
            Err(ServiceError::InternalServerError)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_content_disposition() {
        let input = "form-data; name=\"image\"; filename=\"IMG-20191019-WA0023.jpg\"";

        let result = parse_content_disposition_to_fileending(Some(input));

        assert_eq!("jpg".to_string(), result.unwrap());
    }
}
*/
