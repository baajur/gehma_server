use crate::errors::ServiceError;
use crate::models::{Pool, PushNotificationListener, User};
use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::error::{BlockingError, ResponseError};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::*;
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

pub(crate) struct WebSocket {
    pool: web::Data<Pool>,
    from: User,
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                println!("Ping {:?}", msg);
                ctx.pong(&msg)
            }
            ws::Message::Text(text) => {
                println!("Text {:?}", text);
                ctx.text(text)
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                use crate::schema::push_notifications_listeners::dsl::{
                    push_notifications_listeners, tele_num_from,
                };

                //Close all listeners
                let conn: &r2d2::PooledConnection<_> = &self.pool.get().unwrap();

                let num = diesel::delete(
                    push_notifications_listeners.filter(tele_num_from.eq(&self.from.tele_num)),
                )
                .execute(conn)
                .map_err(|_err| ServiceError::BadRequest("Cannot close listeners".into()));

                ctx.stop();
            }
            _ => (),
        }
    }
}

fn setup_ws(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<Pool>,
    user_id: String,
) -> Result<HttpResponse, Error> {
    use crate::schema::users::dsl::*;
    let conn: &PgConnection = &pool.get().unwrap();
    let parsed = match Uuid::parse_str(&user_id) {
        Ok(w) => w,
        Err(_) => return Err(ServiceError::BadRequest("Invalid User".into()).into()),
    };

    return users
        .filter(id.eq(parsed))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            if let Some(user) = result.first() {
                let resp = ws::start(
                    WebSocket {
                        pool: pool,
                        from: user.clone(),
                    },
                    &req,
                    stream,
                );

                println!("{:?}", resp);

                Ok(resp)
            } else {
                Err(ServiceError::BadRequest("No user found".into()))
            }
        })?;
}

pub fn ws_route(
    req: HttpRequest,
    info: web::Path<(String)>,
    stream: web::Payload,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let info = info.into_inner();

    setup_ws(req, stream, pool, info).map_err(|w| ServiceError::BadRequest(format!("{}", w)))
}

//TODO send notification
