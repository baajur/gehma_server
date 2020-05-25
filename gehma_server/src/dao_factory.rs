use crate::Pool;

use crate::queries::*;

pub struct DaoFactory(Pool);

impl DaoFactory {
    pub fn new(pool: Pool) -> Self {
        Self(pool)
    }

    pub fn get_user_dao(&self) -> Box<dyn PersistentUserDao> {
        Box::new(PgUserDao {
            pool: self.0.clone(),
        })
    }

    pub fn get_blacklist_dao(&self) -> Box<dyn PersistentBlacklistDao> {
        Box::new(PgBlacklistDao {
            pool: self.0.clone(),
        })
    }

    pub fn get_contacts_dao(&self) -> Box<dyn PersistentContactsDao> {
        Box::new(PgContactsDao {
            pool: self.0.clone(),
        })
    }
}
