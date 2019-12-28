use data_encoding::HEXUPPER;
use ring::digest;

use core::models::{Contact, User, Blacklist};

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    let connection = establish_connection();

    update_user(&connection);
    update_contact(&connection);
    update_blacklist(&connection);
}

fn update_user(connection: &PgConnection) {
    use core::schema::contacts::dsl::{contacts, target_tele_num};
    use core::schema::users::dsl::{hash_tele_num, id, users};

    let mut result = users.load::<User>(connection).unwrap();

    for user in result.into_iter() {
        let target = users.filter(id.eq(user.id));

        diesel::update(target)
            .set(
                (hash_tele_num.eq(Some(HEXUPPER.encode(
                    digest::digest(&digest::SHA256, user.tele_num.as_bytes()).as_ref(),
                )))),
            )
            .execute(connection)
            .unwrap();
    }
}

fn update_contact(connection: &PgConnection) {
    use core::schema::contacts::dsl::{contacts, from_id, target_hash_tele_num, target_tele_num};

    let mut result = contacts.load::<Contact>(connection).unwrap();

    for contact in result.into_iter() {
        let target = contacts.filter(
            from_id
                .eq(contact.from_id)
                .and(target_tele_num.eq(contact.target_tele_num.clone())),
        );

        diesel::update(target)
            .set(
                (target_hash_tele_num.eq(Some(HEXUPPER.encode(
                    digest::digest(&digest::SHA256, contact.target_tele_num.as_bytes()).as_ref(),
                )))),
            )
            .execute(connection)
            .unwrap();
    }
}

fn update_blacklist(connection: &PgConnection) {
    use core::schema::blacklist::dsl::{blacklist, id, blocker, blocked, hash_blocker, hash_blocked};

    let mut result = blacklist.load::<Blacklist>(connection).unwrap();

    for b in result.into_iter() {
        let target = blacklist.filter(
           id.eq(b.id)
        );

        diesel::update(target)
            .set(
                (hash_blocked.eq(Some(HEXUPPER.encode(
                    digest::digest(&digest::SHA256, b.blocked.as_bytes()).as_ref(),
                )))),
            )
            .execute(connection)
            .unwrap();

        diesel::update(target)
            .set(
                (hash_blocker.eq(Some(HEXUPPER.encode(
                    digest::digest(&digest::SHA256, b.blocker.as_bytes()).as_ref(),
                )))),
            )
            .execute(connection)
            .unwrap();


    }
}
