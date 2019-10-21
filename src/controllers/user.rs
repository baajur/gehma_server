use crate::Pool;
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error::BlockingError, error::PayloadError, web, HttpResponse};
use core::errors::ServiceError;
use core::models::{PhoneNumber, User};
use diesel::{prelude::*, PgConnection};
use futures::future::{err, Either};
use futures::stream::Stream;
use futures::Future;
use uuid::Uuid;

use log::{debug, error, info};
use std::io::Write;

pub fn add(
    _info: web::Path<()>,
    body: web::Json<PostUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/add");
    debug!("{:?}", body);
    //dbg!(&body);
    web::block(move || create_entry(body.into_inner(), pool)).then(|res| match res {
        Ok(user) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(user);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get(
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/get");
    debug!("path {:?}", info);

    web::block(move || get_entry(&info.into_inner(), pool)).then(|res| match res {
        Ok(users) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(users);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn upload_profile_picture(
    info: web::Path<String>,
    multipart: Multipart,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/upload_profile_picture");

    let uid = info.into_inner();
    multipart
        .map_err(|err| {
            error!("Multipart error: {}", err);
            ServiceError::InternalServerError
        })
        .map(move |field| save_file(uid.clone(), field, pool.clone()).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|err| {
            error!("Multipart error: {}", err);
            err
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUser {
    pub tele_num: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub description: String,
    pub led: String,
    pub is_autofahrer: Option<String>,
    pub client_version: String,
}

pub fn update(
    info: web::Path<(String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/user/update");
    debug!("path {:?}", info);
    debug!("data {:?}", data);

    web::block(move || update_user(&info.into_inner(), &data.into_inner(), &pool)).then(|res| {
        match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&user)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

fn create_entry(body: PostUser, pool: web::Data<Pool>) -> Result<User, ServiceError> {
    info!("controllers/user/create_entry");
    debug!("body {:?}", body);

    if !crate::ALLOWED_CLIENT_VERSIONS.contains(&body.client_version.as_str()) {
        error!("Version mismatch. Server does not suppoert client version");
        return Err(ServiceError::BadRequest(format!(
            "Version mismatch. The supported versions are {:?}",
            crate::ALLOWED_CLIENT_VERSIONS
        )));
    }

    let tele = &body.tele_num;
    let country_code = &body.country_code;

    let tele2 = PhoneNumber::my_from(&tele, country_code)?;

    dbg!(&tele2.to_string());

    let user = match crate::queries::user::create_query(
        &tele2,
        &country_code,
        &body.client_version,
        &pool,
    ) {
        Ok(u) => Ok(u),
        Err(ServiceError::AlreadyExists(_)) => {
            crate::queries::user::get_entry_by_tel_query(&tele2, &pool)
        }
        Err(err) => Err(err),
    }?;

    if user.client_version != body.client_version {
        update_user(
            &user.id.to_string(),
            &UpdateUser {
                description: user.description.clone(),
                led: format!("{}", user.led),
                is_autofahrer: Some(format!("{}", user.is_autofahrer)),
                client_version: body.client_version.clone(),
            },
            &pool,
        )?;
    }

    dbg!(&user);

    crate::queries::user::analytics_usage_statistics(&pool, &user)?;

    Ok(user)
}

fn get_entry(uid: &str, pool: web::Data<Pool>) -> Result<User, ::core::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users = crate::queries::user::get_query(parsed, &pool)?;
    dbg!(&users);

    let user = match users.len() {
        0 => Err(ServiceError::BadRequest("No user found".to_string())),
        _ => Ok(users.get(0).unwrap().clone()),
    }?;

    //analytics_usage_statistics(&pool, &user)?; not logging every refresh

    Ok(user)
}

fn update_user(
    uid: &str,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let user = crate::queries::user::update_user_query(parsed, user, &pool)?;

    dbg!(&user);

    crate::queries::user::analytics_user(&pool, &user)?;

    Ok(user)
}

//TODO add verification
fn save_file(
    uid: String,
    field: Field,
    pool: web::Data<Pool>,
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

    debug!("unsanitized_ending {:?}", unsanitized_ending);

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

    debug!("ending {:?}", ending);

    let parsed = match Uuid::parse_str(&uid) {
        Ok(p) => p,
        Err(_e) => {
            error!("uuid is invalid {}", uid);
            return Either::A(err(ServiceError::InternalServerError));
        }
    };

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
