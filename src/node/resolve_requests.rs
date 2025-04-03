pub mod methods {
    use std::collections::HashMap;
    use std::fmt;
    use std::net::TcpStream;
    use std::io::prelude::*;
    use serde_json::json;

    pub enum HTTPResponse {
        OK,
        InvalidMethod,
        BadRequest
    }

    pub struct HTTPRequest {
        method: String,
        path: String,
        http_version: String,
        headers: HashMap<String, String>,
        body: Option<String>
    }

    impl HTTPRequest {
        pub fn new(method:String, path: String, http_version: String, headers: HashMap<String, String>, body: Option<String>) -> HTTPRequest {
            HTTPRequest {
                method,
                path,
                http_version,
                headers,
                body,
            }
        }
    }

    #[derive(Debug)]
    pub enum HTTPParseError {
        InvalidRequestLine,
        MissingFields,
    }
    impl fmt::Display for HTTPParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                HTTPParseError::InvalidRequestLine => write!(f, "Invalid request line"),
                HTTPParseError::MissingFields => write!(f, "Missing required fields"),
            }
        }
    }
    impl std::error::Error for HTTPParseError {}

    
    type Result<T> = std::result::Result<T, HTTPParseError>;

    pub fn return_json(mut stream: &TcpStream, status: HTTPResponse){
        
        let (msg, status_code) = match status {
            HTTPResponse::OK => ("The request was successful".to_owned(), 200),
            HTTPResponse::InvalidMethod => ("Invalid HTTP method".to_owned(), 405),
            HTTPResponse::BadRequest => ("Bad Request".to_owned(), 400)
        };

        let response = json!({
            "msg": msg,
            "status_code": status_code
        });

        let success_json = serde_json::to_string(&response)
            .expect("Couldn't convert the object to json");

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n
{}", 
            success_json.len(),
            success_json
        );

        stream.write_all(response.as_bytes()).unwrap();
    }

    pub fn get(stream: &TcpStream, endpoint: &str) {
        return_json(stream, HTTPResponse::OK);
    }
    pub fn post(stream: &TcpStream, endpoint: &str) {
        return_json(&stream, HTTPResponse::OK);
    }
}