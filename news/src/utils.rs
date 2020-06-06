use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use log::{info, trace};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Connects to postgres and runs the migration
pub(crate) fn connect_pg(database_url: impl Into<String>) -> Pool {
    trace!("Entering connect_pg");

    let manager = ConnectionManager::<PgConnection>::new(database_url.into());
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    info!("Database was setup");

    trace!("Exiting connect_pg");

    pool
}
