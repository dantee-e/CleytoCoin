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
}

use super::methods::{
    Content,
    GETData,
    HTTPRequest,
    HTTPResponse,
    ImageType,
    Method,
    POSTData,
};

mod helpers {
    use super::*;
    use std::path::PathBuf;
    use super::errors::HTTPResponseError;

    const STATIC_FOLDER: &str = "src/node/static/";

    pub type HTTPResult = Result<HTTPResponse, HTTPResponseError>;


    // These wrappers exist because of a bug in rust:
    // https://github.com/rust-lang/rust/issues/63033
    pub type POSTFunc = fn(&POSTData) -> HTTPResult;
    pub type GETFunc = fn(&GETData) -> HTTPResult;
    pub struct GETFuncWrapper(GETFunc);
    pub struct POSTFuncWrapper(POSTFunc);
    pub fn path_not_found() -> HTTPResult {
        Err(HTTPResponseError::InvalidPath(Some(
            "Path not found".to_string(),
        )))
    }
    pub fn method_not_allowed() -> HTTPResult {
        Err(HTTPResponseError::InvalidMethod(None))
    }

    pub trait Handler {
        fn call(&self, request: &HTTPRequest) -> HTTPResult;
    }

    // Implement the trait for GETFunc
    impl Handler for GETFunc {
        fn call(&self, request: &HTTPRequest) -> HTTPResult {
            match request.get_method() {
                Method::GET(data) => self(&data),
                _ => method_not_allowed(),
            }
        }
    }

    // Implement the trait for POSTFunc
    impl Handler for POSTFunc {
        fn call(&self, request: &HTTPRequest) -> HTTPResult {
            match request.get_method() {
                Method::POST(data) => self(&data),
                _ => method_not_allowed(),
            }
        }
    }

    impl Handler for GETFuncWrapper {
        fn call(&self, request: &HTTPRequest) -> HTTPResult {
            match request.get_method() {
                Method::GET(data) => self.0(&data),
                _ => method_not_allowed(),
            }
        }
    }

    impl Handler for POSTFuncWrapper {
        fn call(&self, request: &HTTPRequest) -> HTTPResult {
            match request.get_method() {
                Method::POST(data) => self.0(&data),
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
        Ok(HTTPResponse::OK(Some(Content::JSON(
            json
        ))))
    }

    pub fn post(request: HTTPRequest, f: POSTFunc) -> HTTPResult {
        let method = request.get_method();
        if let Method::POST(data) = method {
            f(data)
        } else {
            Err(HTTPResponseError::InvalidMethod(None))
        }
    }
    pub fn get(request: HTTPRequest, f: GETFunc) -> HTTPResult {
        if let Method::GET(data) = request.get_method() {
            f(data)
        } else {
            Err(HTTPResponseError::InvalidMethod(None))
        }
    }
    pub fn get_post(request: HTTPRequest, get: GETFunc, post: POSTFunc) -> HTTPResult {
        match request.get_method() {
            Method::POST(data) => post(data),
            Method::GET(data) => get(data),
        }
    }
}


pub mod endpoints {
    use super::*;
    use super::helpers::{return_html, return_image, HTTPResult, Handler, GETFunc, POSTFunc, method_not_allowed, path_not_found, return_json};
    use super::errors::{
        HTTPResponseError
    };

    use std::collections::HashMap;
    use chrono::Utc;
    use crate::chain::transaction::{
        Transaction, TransactionDeserializeError, TransactionValidationError,
    };
    use serde_json::json;

    pub fn index(_: &GETData) -> HTTPResult {
        return_html("index.html")
    }

    pub fn submit_transaction(data: &POSTData) -> HTTPResult {
        let body = data.body.clone().unwrap();
        let transaction: Transaction = match Transaction::deserialize(body) {
            Ok(tx) => tx,
            Err(e) => {
                return match e {
                    TransactionDeserializeError::InsufficientFunds => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                    TransactionDeserializeError::MalformedTransaction => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                    TransactionDeserializeError::SerdeError(_) => {
                        Err(HTTPResponseError::InvalidBody(None))
                    }
                }
            }
        };

        match transaction.verify() {
            Ok(()) => {}
            Err(e) => {
                return match e {
                    TransactionValidationError::OpenSSLError(_) => {
                        Err(HTTPResponseError::InternalServerError(Some(
                            "Error in the OpenSSL library when verifying a transaction"
                                .to_string(),
                        )))
                    }
                    TransactionValidationError::ValidationError => {
                        Err(HTTPResponseError::BadRequest(Some(
                            "Transaction submitted with \
                        invalid signature"
                                .to_string(),
                        )))
                    }
                }
            }
        };

        Ok(HTTPResponse::OK(Some(Content::JSON(json!({
            "msg": "The transaction was added to the pool.",
            "status_code": "200"
        })))))
    }

    pub fn favicon(_: &GETData) -> HTTPResult {
        return_image("fav.ico", ImageType::ICO)
    }

    pub fn status(_: &GETData) -> HTTPResult {
        return_json(json!({
            "status": "Fodeline",
            "blockHeight": 123456,
            "peers": 8,
            "timestamp": Utc::now()
        }))
    }

    pub fn resolve_endpoint(mut request: HTTPRequest) -> Result<Option<String>, Option<String>> {
        /*
        TODO: This creates the endpoints var every time the resolve_endpoints function runs,
         which is very inefficient. We should move the creation of the endpoints var to the
         initialization of the program, and pass it around as a parameter to the functions that
         need it
         */

        fn curry_add_endpoint<'a, 'b>(
            endpoints: &'b mut HashMap<&'a str, HashMap<&'a str, Box<dyn Handler>>>
        ) -> impl FnMut(&'a str, Option<GETFunc>, Option<POSTFunc>) + 'b {
            |path: &'a str,
             get: Option<fn(&GETData) -> HTTPResult>,
             post: Option<fn(&POSTData) -> HTTPResult>
            | {
                let mut methods: HashMap<&'a str, Box<dyn Handler>> = HashMap::new();
                if let Some(get) = get {
                    methods.insert("GET", Box::new(get) as Box<dyn Handler>);
                }
                if let Some(post) = post {
                    methods.insert("POST", Box::new(post) as Box<dyn Handler>);
                }

                endpoints.insert(path, methods);
            }
        }




        let mut endpoints: HashMap<&str, HashMap<&str, Box<dyn Handler>>> = HashMap::new();


        {
            let mut add_endpoints = curry_add_endpoint(&mut endpoints);
            add_endpoints("/", Some(index), None);
        }
        

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

        match r {
            Ok(value) => {
                request.response(value);
                Ok(Some(format!(
                    "Request {} to path {} was successful",
                    method,
                    path.to_str().unwrap()
                )))
            }
            Err(e) => match e {
                HTTPResponseError::InvalidMethod(log) => {
                    request.response(HTTPResponse::InvalidMethod);
                    Err(log)
                }
                HTTPResponseError::InvalidPath(log) => {
                    request.response(HTTPResponse::BadRequest);
                    Err(log)
                }
                HTTPResponseError::InvalidBody(log) => {
                    request.response(HTTPResponse::BadRequest);
                    Err(log)
                }
                HTTPResponseError::InternalServerError(log) => {
                    request.response(HTTPResponse::InternalServerError);
                    Err(log)
                }
                HTTPResponseError::BadRequest(log) => {
                    request.response(HTTPResponse::BadRequest);
                    Err(log)
                }
            },
        }
    }
}