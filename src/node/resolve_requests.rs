pub mod endpoints {
    mod errors {
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
                    _ => todo!(),
                }
            }
        }
        impl std::error::Error for HTTPResponseError {}
    }
    mod helpers {
        use std::path::PathBuf;
        use crate::node::resolve_requests::endpoints::errors::HTTPResponseError;
        use crate::node::resolve_requests::methods::{Content, GETData, HTTPRequest, HTTPResponse, Method, POSTData};

        const STATIC_FOLDER: &str = "src/node/static/";

        pub type HTTPResult = Result<HTTPResponse, HTTPResponseError>;

        pub type POSTFunc = fn(&POSTData) -> HTTPResult;
        pub type GETFunc = fn(&GETData) -> HTTPResult;

        pub fn path_not_found() -> HTTPResult{
            Err(HTTPResponseError::InvalidPath(Some("Path not found".to_string())))
        }
        pub fn method_not_allowed() -> HTTPResult{
            Err(HTTPResponseError::InvalidMethod(None))
        }

        pub trait Handler { fn call(&self, request: &HTTPRequest) -> HTTPResult; }

        // Implement the trait for GETFunc
        impl Handler for GETFunc {
            fn call(&self, request: &HTTPRequest) -> HTTPResult {
                match request.get_method() {
                    Method::GET(data) => self(&data),
                    _ => method_not_allowed()
                }
            }
        }

        // Implement the trait for POSTFunc
        impl Handler for POSTFunc {
            fn call(&self, request: &HTTPRequest) -> HTTPResult {
                match request.get_method() {
                    Method::POST(data) => self(&data),
                    _ => method_not_allowed()
                }
            }
        }

        pub fn return_image(path: &str) -> HTTPResult {
            Ok(HTTPResponse::OK(Some(Content::Image(PathBuf::from(STATIC_FOLDER.to_owned() + path)))))
        }
        pub fn return_html(path: &str) -> HTTPResult {
            Ok(HTTPResponse::OK(Some(Content::HTML(PathBuf::from(STATIC_FOLDER.to_owned() + path)))))
        }

        pub fn post(request: HTTPRequest, f: POSTFunc) -> HTTPResult {
            let method = request.get_method();
            if let Method::POST(data) = method {
                f(data)
            }
            else {
                Err(HTTPResponseError::InvalidMethod(None))
            }
        }
        pub fn get(request: HTTPRequest, f: GETFunc) -> HTTPResult {
            if let Method::GET(data) = request.get_method() {
                f(data)
            }
            else {
                Err(HTTPResponseError::InvalidMethod(None))
            }
        }
        pub fn get_post(
            request: HTTPRequest,
            get: GETFunc,
            post: POSTFunc
        ) -> HTTPResult {
            match request.get_method() {
                Method::POST(data) => post(data),
                Method::GET(data) => get(data)
            }
        }
    }

    use std::collections::HashMap;
    use super::methods::{HTTPRequest, HTTPResponse, Method};
    use helpers::*;

    mod endpoints {
        use crate::chain::transaction::Transaction;
        use crate::node::resolve_requests::endpoints::errors::HTTPResponseError;
        use crate::node::resolve_requests::endpoints::helpers::{return_html, return_image, HTTPResult};
        use crate::node::resolve_requests::methods::{GETData, POSTData};

        pub fn index(data: &GETData) -> HTTPResult {
            return_html("index.html")
        }

        pub fn submit_transaction(data: &POSTData) -> HTTPResult {
            let body = data.body.clone().unwrap();
            let json: Transaction = match serde_json::from_str(body.as_str()) {
                Ok(json) => json,
                Err(_) => {
                    return Err(HTTPResponseError::InvalidBody(None));
                }
            };
            Err(HTTPResponseError::InternalServerError(Some("Something went wrong on the submit_transaction function".parse().unwrap())))

        }

        pub fn favicon(data: &GETData) -> HTTPResult {
            return_image("favicon.ico")
        }
    }
    use endpoints::*;
    use crate::node::resolve_requests::endpoints::errors::HTTPResponseError;

    pub fn resolve_endpoint(mut request: HTTPRequest) -> Result<Option<String>, Option<String>> {
        /*
        TODO: This creates the endpoints var every time the resolve_endpoints function runs,
         which is very inefficient. We should move the creation of the endpoints var to the
         initialization of the program, and pass it around as a parameter to the functions that
         need it
         */

        fn handle_error(request: &mut HTTPRequest, response: HTTPResponse, log: Option<&str>) -> Result<(), String> {
            request.response(response);
            Err(log.map(String::from).unwrap_or_default())
        }

        fn add_endpoint<'a>(
            path: &'a str,
            endpoints: &mut HashMap<&'a str, HashMap<&str, Box<dyn Handler>>>,
            get: Option<GETFunc>,
            post: Option<POSTFunc>
        ) {
            let mut methods: HashMap<&str, Box<dyn Handler>> = HashMap::new();

            if let Some(get) = get {
                methods.insert("GET", Box::new(get) as Box<dyn Handler>);
            }
            if let Some(post) = post {
                methods.insert("POST", Box::new(post) as Box<dyn Handler>);
            }

            endpoints.insert(path, methods);

        }
        

        let mut endpoints: HashMap<&str, HashMap<&str, Box<dyn Handler>>> = HashMap::new();

        add_endpoint("/", &mut endpoints, Some(index), None);
        add_endpoint("/favicon.ico", &mut endpoints, Some(favicon), None);


        let (path, method) = match request.get_method() {
            Method::GET(data) => (data.path.clone(), "GET"),
            Method::POST(data) => (data.path.clone(), "POST"),
        };

         let r = match endpoints.get(path.to_str().unwrap()) {
            Some(methods) => match methods.get(method) {
                Some(handler) => handler.call(&request),
                None => method_not_allowed(),
            },
            None => path_not_found(),
        };
        
        let log = String::new();
        match r {
            Ok(value) => {request.response(value); Ok(Some("Success".to_string()))}
            Err(e) => {
                match e {
                    HTTPResponseError::InvalidMethod(log) => {
                        request.response(HTTPResponse::InvalidMethod);
                        Err(log)
                    },
                    HTTPResponseError::InvalidPath(log) => {
                        request.response(HTTPResponse::BadRequest);
                        Err(log)
                    },
                    HTTPResponseError::InvalidBody(log) => {
                        request.response(HTTPResponse::BadRequest);
                        Err(log)
                    },
                    HTTPResponseError::InternalServerError(log) => {
                        request.response(HTTPResponse::InternalServerError);
                        Err(log)
                    },
                    HTTPResponseError::BadRequest(log) => {
                        request.response(HTTPResponse::BadRequest);
                        Err(log)
                    },
                }
            }
        }
    }
}

pub mod methods {
    use core::panic;
    use std::collections::HashMap;
    use std::{fmt, fs};
    use std::net::TcpStream;
    use std::io::prelude::*;
    use std::path::{Path, PathBuf};
    use serde_json::json;


    pub enum Content {
        HTML(PathBuf),    // path to HTML file
        JSON(serde_json::Value),
        Image(PathBuf),   // path to image file
        PlainText(String),
    }
    pub enum HTTPResponse {
        OK(Option<Content>),
        InvalidMethod,
        BadRequest,
        InternalServerError,
    }


    #[derive(Debug, Clone)]
    pub struct GETData {
        pub path: PathBuf
    }
    #[derive(Debug, Clone)]
    pub struct POSTData {
        pub path: PathBuf,
        pub body: Option<String>
    }

    #[derive(Debug, Clone)]
    pub enum Method {
        GET(GETData),
        POST(POSTData)
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

    #[derive(Debug)]
    pub struct HTTPRequest {
        stream: Option<TcpStream>,
        pub headers: HashMap<String, String>,
        method: Method,
        http_version: String,
    }

    impl HTTPRequest {
        pub fn new(
            stream: Option<TcpStream>, 
            method: String, path: PathBuf, 
            http_version: String, headers: 
            HashMap<String, String>, 
            body: Option<String>
        ) -> HTTPRequest {
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

        pub fn get_method(&self) -> &Method { &self.method }

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
                HTTPResponse::InternalServerError => {
                    if accept.unwrap_or("").contains("text/html") {
                        Ok(Response::new(500, "text/html", fs::read("static/500.html").unwrap_or_else(|_| b"500 Internal Server Error".to_vec())))
                    } else {
                        Ok(Response::new(500, "text/plain", b"500 Internal Server Error".to_vec()))
                    }
                }
            }
        }
        
        pub fn response(&mut self, status: HTTPResponse) {
            let accept = self.headers.get("Accept").map(|s| s.as_str());
            let resp = Self::make_response(status, accept).unwrap();
            if let Err(e) = self.stream.as_mut().unwrap().write_all(&resp.to_bytes()){
                eprintln!("Error writing response to stream: {}", e);
            }

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




}