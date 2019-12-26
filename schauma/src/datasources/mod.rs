use chrono::prelude::*;
use core::errors::SchaumaError;
use core::models::schauma::DatasourceGenericEvent;

pub mod testing;

pub type City = String;
pub type MyDateTime = NaiveDateTime;

use crate::Pool;
use std::sync::Arc;

pub struct EventDatasourceWrapper {
    pub service: Box<dyn EventDatasource>,
}

pub trait EventDatasource: Send + Sync {
    fn get_events(
        &self,
        pool: &Arc<Pool>,
        city: &City,
        opening: MyDateTime,
        closing: Option<MyDateTime>,
    ) -> Result<Vec<DatasourceGenericEvent>, SchaumaError>;
}
