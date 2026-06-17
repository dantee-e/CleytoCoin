use std::fmt;

#[derive(Debug)]
pub enum HTTPResponseError {
    InvalidMethod(Option<String>),
    InvalidPath(Option<String>),
    InvalidBody(Option<String>),
    InternalServerError(Option<String>),
    BadRequest(Option<String>),
    ResourceNotFound(Option<String>),
}
impl fmt::Display for HTTPResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HTTPResponseError::InvalidMethod(log) => match log {
                None => {
                    write!(f, "Invalid Method")
                }
                Some(log) => {
                    write!(f, "Invalid Method: {}", log)
                }
            },
            HTTPResponseError::InvalidPath(log) => match log {
                None => {
                    write!(f, "Invalid Path")
                }
                Some(log) => {
                    write!(f, "Invalid Path: {}", log)
                }
            },
            HTTPResponseError::InvalidBody(log) => match log {
                None => {
                    write!(f, "Invalid Body")
                }
                Some(log) => {
                    write!(f, "Invalid Body: {}", log)
                }
            },
            HTTPResponseError::InternalServerError(log) => match log {
                None => {
                    write!(f, "Internal Server Error")
                }
                Some(log) => {
                    write!(f, "Internal Server Error: {}", log)
                }
            },
            HTTPResponseError::BadRequest(log) => match log {
                None => {
                    write!(f, "Bad Request")
                }
                Some(log) => {
                    write!(f, "Bad Request: {}", log)
                }
            },
            HTTPResponseError::ResourceNotFound(log) => match log {
                None => {
                    write!(f, "Bad Request")
                }
                Some(log) => {
                    write!(f, "Bad Request: {}", log)
                }
            },
        }
    }
}

impl From<openssl::error::Error> for HTTPResponseError {
    fn from(_: openssl::error::Error) -> Self {
        HTTPResponseError::InvalidBody(Some("Invalid cryptographic information".to_string()))
    }
}
impl From<openssl::error::ErrorStack> for HTTPResponseError {
    fn from(_: openssl::error::ErrorStack) -> Self {
        HTTPResponseError::InvalidBody(Some("Invalid cryptographic information".to_string()))
    }
}

impl std::error::Error for HTTPResponseError {}
