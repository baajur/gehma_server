use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::run_pending_migrations;

use log::{info, trace, error};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connects to postgres and runs the migration
pub(crate) fn connect_pg(database_url: impl Into<String>) -> Pool {
    trace!("Entering connect_pg");

    let manager = ConnectionManager::<PgConnection>::new(database_url.into());
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    info!("Database was setup");

    let connection: &PgConnection = &pool.get().unwrap();

    info!("Running pending migrations...");

    let res = run_pending_migrations(connection); //.expect("cannot run pending migrations");

    if let Err(err) = res {
        error!("Cannot run migrations {}", err);
    }

    trace!("Exiting connect_pg");

    pool
}
