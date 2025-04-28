


pub mod endpoints {
    use crate::chain::transaction::Transaction;

    use super::methods::{HTTPRequest, HTTPResponse, Method};

    fn path_not_found() -> Result<String, String>{
        Err(String::from("Someone tried to access a nonexistent path"))
    }

    fn index(mut request: HTTPRequest) -> Result<String, String> {

        request.response(HTTPResponse::OK);
        Ok(String::from("Index.html returned to client"))
    }

    fn submit_transaction(mut request: HTTPRequest) -> Result<String, String> {
        if let Method::GET(_) = request.get_method() {
            request.response(HTTPResponse::InvalidMethod);
            return Err(String::from("Method not supported on function submit_transaction"));
        }

        if let Method::POST(data) = request.get_method() {
            let json: Transaction = match serde_json::from_str(&data.body.unwrap()){
                Ok(json) => json,
                Err(_) => {
                    request.response(HTTPResponse::BadRequest);
                    return Err(String::from("Error extracting transaction from POST").to_string());
                }

            };
            request.response(HTTPResponse::OK);
            return Ok(format!("Received transaction: {}", json.to_string()))
        }

        Err(String::from("Something went wrong on the submit_transaction function"))

    }

    fn favicon(mut request: HTTPRequest) -> Result<String, String> {
        if let Method::GET(_) = request.get_method() {
            // TODO implement the happy path
            return Err(String::from("Favicon"));
        }
    }

    pub fn resolve_endpoint(request: HTTPRequest) -> Result<String, String> {
        match request.get_method() {
            Method::GET(data) => {
                match data.path.as_str() {
                    "/" => index(request),
                    "/favicon.ico" => favicon(request),
                    _ => path_not_found(),

                }
            },
            Method::POST(data) => {
                match data.path.as_str() {
                    "/" => index(request),
                    _ => path_not_found(),
                }
            },
        }

    }
}

pub mod methods {
    use core::panic;
    use std::collections::HashMap;
    use std::{fmt, fs};
    use std::net::TcpStream;
    use std::io::prelude::*;
    use serde_json::json;


    pub enum Content {
        HTML(String),    // path to HTML file
        JSON(serde_json::Value),
        Image(String),   // path to image file
        PlainText(String),
    }
    pub enum HTTPResponse {
        OK(Option<Content>),
        InvalidMethod,
        BadRequest,
        Favicon
    }


    #[derive(Debug, Clone)]
    pub struct GETData {
        pub path: String
    }
    #[derive(Debug, Clone)]
    pub struct POSTData {
        pub path: String,
        pub body: Option<String>
    }

    #[derive(Debug, Clone)]
    pub enum Method {
        GET(GETData),
        POST(POSTData)
    }

    

    #[derive(Debug)]
    pub struct HTTPRequest {
        stream: Option<TcpStream>,
        pub headers: HashMap<String, String>,
        method: Method,
        http_version: String,
    }

    struct Response {
        status: u16,
        content_type: &'static str,
        body: Vec<u8>,
    }

    impl Response {
        fn to_bytes(&self) -> Vec<u8> {
            let mut v = Vec::new();
            let header = format!(
                "HTTP/1.1 {} {}\r\n\
                 Content-Type: {}\r\n\
                 Content-Length: {}\r\n\r\n",
                self.status,
                match self.status {
                    200 => "OK",
                    400 => "Bad Request",
                    405 => "Method Not Allowed",
                    _   => "Unknown",
                },
                self.content_type,
                self.body.len(),
            );
            v.extend_from_slice(header.as_bytes());
            v.extend_from_slice(&self.body);
            v
        }
        fn new(status: u16, content_type: &'static str, body: Vec<u8>) -> Self {
            Self {
                status,
                content_type,
                body
            }
        }
    }

    impl HTTPRequest {
        pub fn new(stream: Option<TcpStream>, method: String, path: String, http_version: String, headers: HashMap<String, String>, body: Option<String>) -> HTTPRequest {
            HTTPRequest {
                stream,
                method: match method.as_str() {
                    "GET" => Method::GET(GETData {path}),
                    "POST" => Method::POST(POSTData {path, body}),
                    _ => panic!("Unavailable method")
                },
                http_version,
                headers
                
            }
        }

        pub fn set_stream(&mut self, stream: TcpStream) { self.stream = Some(stream) }

        pub fn get_method(&self) -> Method { self.method.clone() }



        fn make_response(status: HTTPResponse, accept: Option<&str>) -> std::io::Result<Response> {

            fn response_ok_content(content: Content) -> std::io::Result<Response> {
                match content {
                    Content::HTML(path) => Ok(Response {
                        status: 200,
                        content_type: "text/html",
                        body: fs::read(path)?,
                    }),
                    Content::JSON(value) => Ok(Response {
                        status: 200,
                        content_type: "application/json",
                        body: serde_json::to_vec(&value)?,
                    }),
                    Content::Image(path) => Ok(Response {
                        status: 200,
                        content_type: "image/png",
                        body: fs::read(path)?,
                    }),
                    Content::PlainText(text) => Ok(Response {
                        status: 200,
                        content_type: "text/plain",
                        body: text.into_bytes(),
                    }),
                }
            }
            fn response_ok_no_content(a: &str) -> std::io::Result<Response> {
                if a.contains("text/html") {
                    return Ok(Response::new(200, "text/html", fs::read("static/200.html").unwrap_or("NOT FOUND".into())));
                }
                
                else if a.contains("application/json") {
                    let j = json!({ "msg": "OK", "status": 200 });
                    return Ok(Response::new(200, "application/json", serde_json::to_vec(&j)?))
                }
                
                Ok(Response::new(200, "text", "Success".into()))
            }

            match status {
                HTTPResponse::OK(content_opt) => {
                    if let Some(content) = content_opt {
                        response_ok_content(content)
                    } else {
                        response_ok_no_content(accept.unwrap_or("text"))
                    }
                }
                HTTPResponse::InvalidMethod => {
                    if accept.unwrap_or("").contains("text/html") {
                        Ok(Response::new(405, "text/html", fs::read("static/405.html").unwrap_or_else(|_| b"405 Invalid Method".to_vec())))
                    } else {
                        Ok(Response::new(405, "text/plain", b"405 Invalid Method".to_vec()))
                    }
                }
                HTTPResponse::BadRequest => {
                    if accept.unwrap_or("").contains("text/html") {
                        Ok(Response::new(400, "text/html", fs::read("static/400.html").unwrap_or_else(|_| b"400 Bad Request".to_vec())))
                    } else {
                        Ok(Response::new(400, "text/plain", b"400 Bad Request".to_vec()))
                    }
                }
                HTTPResponse::Favicon => {
                    Ok(Response::new(200, "image/x-icon", fs::read("static/favicon.ico").unwrap_or_else(|_| b"".to_vec())))
                }
            }.expect("TODO: panic message");

            Ok(Response::new(500, "text/html", fs::read("static/500.html").unwrap_or_else(|_| b"500 Internal Server Error".to_vec())))
        }
        
        pub fn response(&mut self, status: HTTPResponse) -> std::io::Result<()> {
            let accept = self.headers.get("Accept").map(|s| s.as_str());
            let resp = Self::make_response(status, accept)?;
            self.stream.as_mut().unwrap().write_all(&resp.to_bytes())
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