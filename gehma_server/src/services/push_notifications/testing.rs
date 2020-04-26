use super::*;
use core::errors::ServiceError;

pub struct TestingNotificationService;

impl NotificationServiceTrait for TestingNotificationService {
    fn push(&self, _: Vec<(String, FirebaseToken)>) -> Result<(), ServiceError> {
        Ok(())
    }
}
