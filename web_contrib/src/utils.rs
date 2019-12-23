use serde::{Deserialize};
use actix_web::http::header::*;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub access_token: String,
}

pub fn set_response_headers(response: &mut actix_web::HttpResponse) {

    response.headers_mut().insert(
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    response.headers_mut().insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("script-src 'self'"),
    );

    response
        .headers_mut()
        .insert(X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN"));

    response
        .headers_mut()
        .insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

    response.headers_mut().insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
}

#[derive(Debug, Clone, Eq)]
pub(crate) struct Version {
    minor: usize,
    major: usize,
    patch: usize,
}

impl Version {
    #[allow(dead_code)]
    pub fn new(raw: impl Into<String>) -> Result<Self, ()> {
        let s = raw.into();

        let splitted = s
            .split('.')
            .map(|w| w.parse().expect("Version can only contain numbers"))
            .collect::<Vec<_>>();

        let (major, minor, patch) = match splitted.len() {
            2 => {
                let major = splitted[0];
                let minor = splitted[1];

                (major, minor, 0)
            }
            3 => {
                let major = splitted[0];
                let minor = splitted[1];
                let patch = splitted[2];

                (major, minor, patch)
            }
            _ => {
                return Err(());
            }
        };

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}

use std::cmp::Ordering;
impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        if self.major < other.major {
            Ordering::Less
        }
        else if self.major == other.major {
            if self.minor < other.minor {
                Ordering::Less
            }
            else if self.minor == other.minor {
                Ordering::Equal
            }
            else {
                Ordering::Greater
            }
            //self.minor.cmp(&other.minor)
        }
        else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        let m = self.major.eq(&other.major);

        if m {
            self.minor.eq(&other.minor)
        }
        else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version1() {
        let v1 = Version::new("1.0");
        let v2 = Version::new("2.0");
    
        assert!(v1 < v2);
    }

    #[test]
    fn test_version2() {
        let v1 = Version::new("1.0");
        let v2 = Version::new("2.0");
    
        assert!(!(v1 > v2));
    }

    #[test]
    fn test_version3() {
        let v1 = Version::new("2.0");
        let v2 = Version::new("2.0");
    
        assert!(v1 == v2);
    }

    #[test]
    fn test_version4() {
        let v1 = Version::new("2.1");
        let v2 = Version::new("2.0");

        assert!(v1 > v2);
    }

    #[test]
    fn test_version5() {
        let v1 = Version::new("2.0.1");
        let v2 = Version::new("2.0");
    
        assert!(v1 == v2);
    }

    #[test]
    fn test_version6() {
        let v1 = Version::new("2.0");
        let v2 = Version::new("2.1");
    
        assert!(v1 != v2);
    }
}
