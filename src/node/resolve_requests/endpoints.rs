use super::super::LOG_LEVEL;
use super::errors::HTTPResponseError;
use super::helpers::{
    method_not_allowed, path_not_found, return_html, return_image, return_json, GETFunc,
    HTTPResult, Handler, POSTFunc,
};
use super::messages::process::{
    check_block_by_block_header, check_transaction_by_transaction_header, process_get_data_block,
    process_get_data_transaction, process_new_block,
};
use super::messages::{GetDataMessage, Message};
use super::methods::{Content, GETData, HTTPRequest, HTTPResponse, ImageType, Method, POSTData};
use crate::chain::transaction::Transaction;
use crate::error_handling::{TransactionDeserializeError, TransactionError};
use crate::node::resolve_requests::messages::process::process_new_node;
use crate::node::NodeState;

use chrono::Utc;
use core::panic;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// pub type POSTFunc = fn(&POSTData, Arc<Mutex<NodeState>>) -> HTTPResult;
// pub type GETFunc = fn(&GETData, Arc<Mutex<NodeState>>) -> HTTPResult;

pub fn index(_: &GETData, _: Arc<Mutex<NodeState>>) -> HTTPResult {
    return_html("index.html")
}

// pub fn send_message(message: Message) -> CleytoResult<()> {}

pub fn submit_transaction(data: &POSTData, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    // Deserializes the transactoibn
    let body = data.body.clone().unwrap();
    let transaction: Transaction = match serde_json::from_str(&body) {
        Ok(tx) => tx,
        Err(_) => return Err(HTTPResponseError::InvalidBody(None)),
    };

    // Check if the funds are enough for the transaction

    match Transaction::check_transaction(&transaction) {
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
    }
    match transaction.verify_signature() {
        Ok(()) => {}
        Err(e) => {
            return match e {
                TransactionError::OpenSSLError(_) => Err(HTTPResponseError::InternalServerError(
                    Some("Error in the OpenSSL library when verifying a transaction".to_string()),
                )),
                TransactionError::ValidationError => Err(HTTPResponseError::BadRequest(Some(
                    "Transaction submitted with \
                    invalid signature"
                        .to_string(),
                ))),
                TransactionError::InsufficientInputs => Err(HTTPResponseError::BadRequest(Some(
                    "Transaction's outputs are bigger that its inputs".to_string(),
                ))),
                // TODO Should move both of those to another error enum, maybe client and server errors
                TransactionError::InsufficientFunds => panic!("Not the server's problem"),
                TransactionError::ConnectionError(_) => panic!("Not the server's problem"),
            };
        }
    };

    state.lock().unwrap().transactions_pool.push(transaction);

    Ok(HTTPResponse::OK(Some(Content::JSON(json!({
        "msg": "The transaction was added to the pool.",
        "status_code": "200"
    })))))
}

pub fn get_transaction_pool(_: &GETData, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    let transaction_pool: Vec<Transaction> = state.lock().unwrap().transactions_pool.clone();
    let response = serde_json::to_value(transaction_pool).unwrap();
    Ok(HTTPResponse::OK(Some(Content::JSON(response))))
}

pub fn favicon(_: &GETData, _: Arc<Mutex<NodeState>>) -> HTTPResult {
    return_image("fav.ico", ImageType::ICO)
}

pub fn status(_: &GETData, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    let state = match state.lock() {
        Ok(guard) => guard,
        Err(_) => panic!("Mutex lock was poisoned in function status on endpoints"),
    };

    return_json(json!({
        "status": state.status,
        "blockHeight": state.chain.get_last_index(),
        "peers": 100000000,
        "timestamp": Utc::now()
    }))
}

pub fn messages(data: &POSTData, state: Arc<Mutex<NodeState>>) -> HTTPResult {
    let body = data.body.clone().unwrap();
    let message: Message =
        serde_json::from_str(&body).map_err(|_| HTTPResponseError::InvalidBody(None))?;

    match message {
        Message::CheckBlock(block_header) => {
            let chain = &state.lock().unwrap().chain;
            check_block_by_block_header(block_header, chain)
        }
        Message::Block(block) => {
            let chain = &mut state.lock().unwrap().chain;
            process_new_block(block, chain)
        }
        Message::NewNode(new_node_message) => {
            let connected_nodes = &mut state.lock().unwrap().connected_nodes;
            process_new_node(new_node_message, connected_nodes)
        }
        Message::KeyRefresh => todo!(),
        Message::CheckTransaction(transaction_header) => {
            let transaction_pool = &state.lock().unwrap().transactions_pool;
            check_transaction_by_transaction_header(transaction_header, transaction_pool)
        }
        Message::GetData(get_data_message) => match get_data_message {
            GetDataMessage::Block(block_header) => {
                let chain = &state.lock().unwrap().chain;
                process_get_data_block(block_header, chain)
            }
            GetDataMessage::Transaction(transaction_header) => {
                let transaction_pool = &state.lock().unwrap().transactions_pool;
                process_get_data_transaction(transaction_header, transaction_pool)
            }
        },
    }
}

pub fn resolve_endpoint(
    state: Arc<Mutex<NodeState>>,
    mut request: HTTPRequest,
) -> Result<Option<String>, Option<String>> {
    /*
    TODO: This creates the endpoints var every time the resolve_endpoints function runs,
    which is inefficient. We should move the creation of the endpoints var to the
    initialization of the program, and pass it around as a parameter to the functions that
    need it
    */

    fn curry_add_endpoint<'a, 'b>(
        endpoints: &'b mut HashMap<&'a str, HashMap<&'a str, Box<dyn Handler>>>,
    ) -> impl FnMut(&'a str, Option<GETFunc>, Option<POSTFunc>) + 'b {
        |path: &'a str, get: Option<GETFunc>, post: Option<POSTFunc>| {
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
    fn initialize_endpoints<'a>() -> HashMap<&'a str, HashMap<&'a str, Box<dyn Handler>>> {
        let mut endpoints: HashMap<&str, HashMap<&str, Box<dyn Handler>>> = HashMap::new();
        {
            let mut add_endpoints = curry_add_endpoint(&mut endpoints);
            add_endpoints("/", Some(index), None);
            add_endpoints("/favicon.ico", Some(favicon), None);
            add_endpoints("/status", Some(status), None);
            add_endpoints("/submit-transaction", None, Some(submit_transaction));
            add_endpoints("/get-transaction-pool", Some(get_transaction_pool), None);
            add_endpoints("/messages", None, Some(messages));
        }
        endpoints
    }

    let endpoints = initialize_endpoints();

    let (path, method) = match request.get_method() {
        Method::GET(data) => (data.path.clone(), "GET"),
        Method::POST(data) => (data.path.clone(), "POST"),
    };

    let r = match endpoints.get(path.to_str().unwrap()) {
        Some(methods) => match methods.get(method) {
            Some(handler) => handler.call(&request, state),
            None => method_not_allowed(Some(path.to_str().unwrap())),
        },
        None => path_not_found(Some(path.to_str().unwrap())),
    };

    match r {
        Ok(value) => {
            request.response(value);
            let path = path.to_str().unwrap();
            // I don't give a single fuck about favicon
            if path == "/favicon.ico" || LOG_LEVEL < 2 {
                return Ok(None);
            }
            Ok(Some(format!(
                "Request {} to path {} was successful",
                method, path
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
            HTTPResponseError::ResourceNotFound(log) => {
                request.response(HTTPResponse::ResourceNotFound);
                Err(log)
            }
        },
    }
}
