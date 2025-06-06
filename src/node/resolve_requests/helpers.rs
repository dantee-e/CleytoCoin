
use crate::node::resolve_requests::errors::HTTPResponseError;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::node::NodeState;
use super::methods::{
    Content, 
    GETData, 
    HTTPRequest, 
    HTTPResponse,
    ImageType,
    Method, 
    POSTData
};

const STATIC_FOLDER: &str = "src/node/static/";

pub type HTTPResult = Result<HTTPResponse, HTTPResponseError>;

pub type POSTFunc = fn(&POSTData, Arc<Mutex<NodeState>>) -> HTTPResult;
pub type GETFunc = fn(&GETData, Arc<Mutex<NodeState>>) -> HTTPResult;
pub fn path_not_found(s: &str) -> HTTPResult {
    Err(HTTPResponseError::InvalidPath(Some(format!(
        "Path {} was not found",
        s
    ))))
}
pub fn method_not_allowed() -> HTTPResult {
    Err(HTTPResponseError::InvalidMethod(None))
}

pub trait Handler {
    fn call(&self, request: &HTTPRequest, state: Arc<Mutex<NodeState>>) -> HTTPResult;
}

// Implement the trait for GETFunc
impl Handler for GETFunc {
    fn call(&self, request: &HTTPRequest, state: Arc<Mutex<NodeState>>) -> HTTPResult {
        match request.get_method() {
            Method::GET(data) => self(&data, state),
            _ => method_not_allowed(),
        }
    }
}

// Implement the trait for POSTFunc
impl Handler for POSTFunc {
    fn call(&self, request: &HTTPRequest, state: Arc<Mutex<NodeState>>) -> HTTPResult {
        match request.get_method() {
            Method::POST(data) => self(&data, state),
            _ => method_not_allowed(),
        }
    }
}

pub fn return_image(path: &str, image_type: ImageType) -> HTTPResult {
    Ok(HTTPResponse::OK(Some(Content::Image(
        PathBuf::from(STATIC_FOLDER.to_owned() + path),
        image_type,
    ))))
}
pub fn return_html(path: &str) -> HTTPResult {
    Ok(HTTPResponse::OK(Some(Content::HTML(PathBuf::from(
        STATIC_FOLDER.to_owned() + path,
    )))))
}
pub fn return_json(json: serde_json::Value) -> HTTPResult {
    Ok(HTTPResponse::OK(Some(Content::JSON(json))))
}

pub fn post(request: HTTPRequest, f: POSTFunc, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    let method = request.get_method();
    if let Method::POST(data) = method {
        f(data, state)
    } else {
        Err(HTTPResponseError::InvalidMethod(None))
    }
}
pub fn get(request: HTTPRequest, f: GETFunc, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    if let Method::GET(data) = request.get_method() {
        f(data, state)
    } else {
        Err(HTTPResponseError::InvalidMethod(None))
    }
}
pub fn get_post(request: HTTPRequest, get: GETFunc, post: POSTFunc, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    match request.get_method() {
        Method::POST(data) => post(data, state),
        Method::GET(data) => get(data, state),
    }
}