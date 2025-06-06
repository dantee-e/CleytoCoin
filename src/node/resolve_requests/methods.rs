use serde_json::json;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::PathBuf;
use std::{fmt, fs};

pub enum ImageType {
    PNG,
    ICO,
    JPEG,
}

pub enum Content {
    HTML(PathBuf), // path to HTML file
    JSON(serde_json::Value),
    Image(PathBuf, ImageType), // path to image file
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
    pub path: PathBuf,
}
#[derive(Debug, Clone)]
pub struct POSTData {
    pub path: PathBuf,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Method {
    GET(GETData),
    POST(POSTData),
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::GET(data) => write!(f, "GET {:?}", data.path),
            Method::POST(data) => write!(f, "POST {:?}", data.path),
        }
    }
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
                _ => "Unknown",
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
            body,
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
        method: String,
        path: PathBuf,
        http_version: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> HTTPRequest {
        HTTPRequest {
            stream,
            method: match method.as_str() {
                "GET" => Method::GET(GETData { path }),
                "POST" => Method::POST(POSTData { path, body }),
                _ => panic!("Unavailable method"),
            },
            http_version,
            headers,
        }
    }

    pub fn set_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream)
    }

    pub fn get_method(&self) -> &Method {
        &self.method
    }

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
                Content::Image(path, img_type) => Ok(Response {
                    status: 200,
                    content_type: match img_type {
                        ImageType::PNG => "image/png",
                        ImageType::ICO => "image/vnd.microsoft.icon",
                        ImageType::JPEG => "image/jpeg",
                    },
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
                return Ok(Response::new(
                    200,
                    "text/html",
                    fs::read("static/200.html").unwrap_or("NOT FOUND".into()),
                ));
            } else if a.contains("application/json") {
                let j = json!({ "msg": "OK", "status": 200 });
                return Ok(Response::new(
                    200,
                    "application/json",
                    serde_json::to_vec(&j)?,
                ));
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
                    Ok(Response::new(
                        405,
                        "text/html",
                        fs::read("static/405.html")
                            .unwrap_or_else(|_| b"405 Invalid Method".to_vec()),
                    ))
                } else {
                    Ok(Response::new(
                        405,
                        "text/plain",
                        b"405 Invalid Method".to_vec(),
                    ))
                }
            }
            HTTPResponse::BadRequest => {
                if accept.unwrap_or("").contains("text/html") {
                    Ok(Response::new(
                        400,
                        "text/html",
                        fs::read("static/400.html").unwrap_or_else(|_| b"400 Bad Request".to_vec()),
                    ))
                } else {
                    Ok(Response::new(
                        400,
                        "text/plain",
                        b"400 Bad Request".to_vec(),
                    ))
                }
            }
            HTTPResponse::InternalServerError => {
                if accept.unwrap_or("").contains("text/html") {
                    Ok(Response::new(
                        500,
                        "text/html",
                        fs::read("static/500.html")
                            .unwrap_or_else(|_| b"500 Internal Server Error".to_vec()),
                    ))
                } else {
                    Ok(Response::new(
                        500,
                        "text/plain",
                        b"500 Internal Server Error".to_vec(),
                    ))
                }
            }
        }
    }

    pub fn response(&mut self, status: HTTPResponse) {
        let accept = self.headers.get("Accept").map(|s| s.as_str());
        let resp = Self::make_response(status, accept).unwrap();
        if let Err(e) = self.stream.as_mut().unwrap().write_all(&resp.to_bytes()) {
            eprintln!("Error writing response to stream: {}", e);
        }
    }
}

impl fmt::Display for HTTPRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the HTTP method and version
        writeln!(f, "Method: {}", self.method)?;
        writeln!(f, "HTTP Version: {}", self.http_version)?;

        // Write the stream status
        writeln!(
            f,
            "Stream: {}",
            if self.stream.is_some() {
                "Connected"
            } else {
                "Disconnected"
            }
        )?;

        // Write the headers
        writeln!(f, "Headers:")?;
        if self.headers.is_empty() {
            writeln!(f, "  (none)")?;
        } else {
            for (key, value) in &self.headers {
                writeln!(f, "  {}: {}", key, value)?;
            }
        }

        Ok(())
    }
}
#[derive(Debug)]
pub enum HTTPParseError {
    InvalidStatusLine,
    InvalidRequestLine,
    MissingFields,
    MissingContentLength,
}
impl fmt::Display for HTTPParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HTTPParseError::InvalidRequestLine => write!(f, "Invalid request line"),
            HTTPParseError::MissingFields => write!(f, "Missing required fields"),
            HTTPParseError::InvalidStatusLine => write!(f, "Invalid status line"),
            HTTPParseError::MissingContentLength => {
                write!(f, "Missing content-length field in headers")
            }
        }
    }
}
impl std::error::Error for HTTPParseError {}
