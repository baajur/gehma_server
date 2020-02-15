use super::*;
use core::errors::SchaumaError;
use core::models::schauma::{DatasourceGenericEvent, Event};

use crate::Pool;
use std::sync::Arc;

pub struct TestingDatasource;

impl EventDatasource for TestingDatasource {
    fn get_events(
        &self,
        pool: &Arc<Pool>,
        city: &City,
        opening: MyDateTime,
        closing: Option<MyDateTime>,
    ) -> Result<Vec<DatasourceGenericEvent>, SchaumaError> {
        let events = vec![Event::generate(
            "Gehma Welcome Party",
            "Welcome all",
            "Austria",
            "Vienna",
            "Hauptstraße 1",
        )];

        Ok(events.into_iter().map(|w| w.into()).collect())
    }
}
