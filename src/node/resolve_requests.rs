
pub mod endpoints {
    use super::methods::{self, HTTPRequest, HTTPResponse};


    fn path_not_found(){

    }

    fn index(mut request: HTTPRequest){
        request.response_json(HTTPResponse::OK);
    }


    pub fn resolve_endpoint(request: HTTPRequest){
        
        if request.get_method() == "GET" {
            match request.get_path().as_str() {
                "/" => index(request),
                _ => path_not_found(),
            }
        }

        else if request.get_method() == "POST" {
            match request.get_path().as_str() {
                "/" => index(request),
                _ => path_not_found(),
            }
        }
        
        
    }
}

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

    #[derive(Debug)]
    pub struct HTTPRequest {
        stream: Option<TcpStream>,
        method: String,
        path: String,
        http_version: String,
        headers: HashMap<String, String>,
        body: Option<String>
    }

    impl HTTPRequest {
        pub fn new(stream: Option<TcpStream>, method:String, path: String, http_version: String, headers: HashMap<String, String>, body: Option<String>) -> HTTPRequest {
            HTTPRequest {
                stream,
                method,
                path,
                http_version,
                headers,
                body,
            }
        }

        pub fn set_stream(&mut self, stream: TcpStream) {
            self.stream = Some(stream);
        }

        pub fn get_method(&self) -> String {
            self.method.clone()
        }
        pub fn get_path(&self) -> String {
            self.path.clone()
        }

        pub fn response_json(&mut self, status: HTTPResponse) {
            let mut stream = self.stream.as_ref().unwrap();
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
    }

    #[derive(Debug)]
    pub enum HTTPParseError {
        InvalidStatusLine,
        InvalidRequestLine,
        MissingFields,
        MissingContentLength
    }
    impl fmt::Display for HTTPParseError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                HTTPParseError::InvalidRequestLine => write!(f, "Invalid request line"),
                HTTPParseError::MissingFields => write!(f, "Missing required fields"),
                HTTPParseError::InvalidStatusLine => write!(f, "Invalid status line"),
                HTTPParseError::MissingContentLength => write!(f, "Missing content-length field in headers"),
            }
        }
    }
    impl std::error::Error for HTTPParseError {}

    

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

    pub fn get(stream: &TcpStream, request: HTTPRequest) {
        println!("{:#?}", request);
        return_json(stream, HTTPResponse::OK);
    }
    pub fn post(stream: &TcpStream, request: HTTPRequest) {
        println!("{:#?}", request);
        return_json(&stream, HTTPResponse::OK);
    }
}