use std::fmt;

#[derive(Debug)]
pub enum HTTPResponseError {
    InvalidMethod(Option<String>),
    InvalidPath(Option<String>),
    InvalidBody(Option<String>),
    InternalServerError(Option<String>),
    BadRequest(Option<String>),
}
impl fmt::Display for HTTPResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // If log adds log, else just prints the type of the enum
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
        }
    }
}
impl std::error::Error for HTTPResponseError {}
