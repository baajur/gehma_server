use chrono::prelude::*;
use core::errors::ServiceError;
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use mockall::*;
use time::Duration;
use uuid::Uuid;
use log::{info, error};

const SESSION_DURATION: i64 = 8;

pub type SessionService = Box<dyn SessionKeyVerification>;

/// `Claims` is encoded the session token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (whom to refer to)
    sub: String,
    /// Expiration date
    exp: i64,
    /// Issue date
    iat: i64,
}

pub struct SessionServicePriv {
    secret: String,
}

impl SessionServicePriv {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

#[automock]
pub trait SessionKeyVerification {
    /// Creates new session key
    fn new_session(&self, id: Uuid) -> (String, Claims);

    /// Returns `true` if `token` is valid
    fn validate(&self, token: String) -> Result<bool, ServiceError>;
}

impl Claims {
    pub fn new(id: Uuid) -> Self {
        Self {
            sub: id.simple().to_string(),
            exp: Utc::now().checked_add_signed(Duration::hours(SESSION_DURATION)).unwrap().timestamp(),
            iat: Utc::now().timestamp(),
        }
    }
}

impl SessionKeyVerification for SessionServicePriv {
    fn new_session(&self, id: Uuid) -> (String, Claims) {
        let claim = Claims::new(id);
        (encode(&Header::default(), &claim, self.secret.as_ref()).unwrap(), claim)
    }

    fn validate(&self, token: String) -> Result<bool, ServiceError> {
        let token = decode::<Claims>(&token, self.secret.as_ref(), &Validation::default());

        match token {
            Ok(_) => Ok(true),
            Err(err) => {
                error!("session {:?}", err);
                info!("Session invalid");
                Ok(false)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_correct() {
        let service = SessionServicePriv::new("my secret".to_string());
        let (token, _) = service.new_session(Uuid::new_v4());

        let c = service.validate(token);
        assert!(c.is_ok());
        assert_eq!(Ok(true), c);
    }

    #[test]
    fn test_session_key_expired() {
        let service = SessionServicePriv::new("my secret".to_string());
        let (token, mut claim) = service.new_session(Uuid::new_v4());

        claim.exp = 0;
        let updated = encode(&Header::default(), &claim, "my secret".as_ref()).unwrap();

        let c = service.validate(updated);
        assert!(c.is_ok());
        assert_eq!(Ok(false), c);
    }

    #[test]
    fn test_session_key_different_secrets() {
        let service = SessionServicePriv::new("my secret".to_string());
        let (mut token, mut _claim) = service.new_session(Uuid::new_v4());
        
        let service2 = SessionServicePriv::new("other secret".to_string());

        let c = service2.validate(token);
        assert_eq!(Ok(false), c);
    }
}
