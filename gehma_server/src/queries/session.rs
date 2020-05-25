use core::errors::ServiceError;
use uuid::Uuid;
use mockall::*;

#[automock]
pub trait PersistentSessionDao {
    fn set_new_session(&self, id: &Uuid, session: &str, expire: Option<i64>) -> Result<(), ServiceError>;
    
    fn clear_session(&self, id: &Uuid) -> Result<(), ServiceError>;
}
