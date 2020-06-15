use data_encoding::HEXUPPER;
use ring::digest;

use core::models::dao::*;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

const ACCESS_TOKEN_LENGTH : usize = 32;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    let connection = establish_connection();

    update_user(&connection);
}

fn update_user(connection: &PgConnection) {
    use core::schema::users::dsl::{users, access_token, id};

    let mut result = users.load::<UserDao>(connection);

    if let Err(ref err) = result {
        eprintln!("err {}", err);
    }

    for user in result.unwrap().into_iter() {
        let target = users.filter(id.eq(user.id));

        let token = core::utils::generate_random_string(ACCESS_TOKEN_LENGTH);

        diesel::update(target)
            .set(
                (access_token.eq(token)),
            )
            .execute(connection)
            .unwrap();
    }
}
