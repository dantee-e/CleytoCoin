pub mod logger;
mod resolve_requests;
mod thread_pool;
pub mod ui;
mod utils;

use crate::chain::{transaction::Transaction, Chain};
use core::panic;
use resolve_requests::{
    endpoints::resolve_endpoint,
    methods::{HTTPParseError, HTTPRequest},
};
use std::time::Duration;
use thread_pool::custom_thread_pool::ThreadPool;

use std::path::PathBuf;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Receiver, Arc, Mutex},
    thread,
};

pub struct Node {
    chain: Chain,
    transactions_list: Vec<Transaction>,
    logger: Arc<logger::Logger>,
}

use crate::node::logger::Logger;
use once_cell::sync::Lazy;

static NUMBER_OF_THREADS_IN_THREAD_POOL: Lazy<usize> = Lazy::new(num_cpus::get);

impl Node {
    // these configurations should be moved to a file
    pub const DEFAULT_PORT: u16 = 9473;
    pub const REFRESH_RATE_SERVER_IN_MS: u64 = 50;

    pub fn new(chain: Chain, logger: Arc<Logger>) -> Node {
        num_cpus::get();

        Node {
            chain,
            transactions_list: Vec::new(),
            logger,
        }
    }

    fn parse_http_request<R: Read>(
        mut buf_reader: BufReader<R>,
    ) -> Result<HTTPRequest, HTTPParseError> {
        let mut http_headers: HashMap<String, String> = HashMap::new();
        let http_body: Option<String>;

        let mut line = String::new();

        // reading status_line
        let status_line: String;

        match buf_reader.read_line(&mut line) {
            Ok(n) if (n > 0) => n,
            Ok(_) => return Err(HTTPParseError::InvalidStatusLine),
            Err(_) => return Err(HTTPParseError::InvalidStatusLine),
        };

        status_line = line.trim().to_string();

        let mut tokens = status_line.split(' ');
        let (method, path, http_version) = (
            tokens
                .next()
                .ok_or(HTTPParseError::InvalidRequestLine)?
                .to_string(),
            tokens
                .next()
                .ok_or(HTTPParseError::InvalidRequestLine)?
                .to_string(),
            tokens
                .next()
                .ok_or(HTTPParseError::InvalidRequestLine)?
                .to_string(),
        );

        // reading headers
        loop {
            line.clear();

            if let Err(_) = buf_reader.read_line(&mut line) {
                return Err(HTTPParseError::InvalidRequestLine);
            }

            let line = line.trim_end().to_string();

            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(":") {
                http_headers.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                return Err(HTTPParseError::InvalidRequestLine);
            };
        }

        // If method is GET, return before trying to read the body
        if method == "GET" {
            return Ok(HTTPRequest::new(
                None,
                method,
                PathBuf::from(path),
                http_version,
                http_headers,
                None,
            ));
        }

        // getting content_length from headers
        let content_length = match http_headers.get("content-length") {
            Some(value) => match value.parse::<usize>() {
                Ok(length) => length,
                Err(_) => {
                    return Err(HTTPParseError::MissingContentLength);
                }
            },
            None => {
                return Err(HTTPParseError::MissingContentLength);
            }
        };

        // reading body
        let mut body = vec![0; content_length];
        if let Err(e) = buf_reader.read_exact(&mut body) {
            eprintln!("Error reading body: {}", e);
            return Err(HTTPParseError::InvalidRequestLine);
        }

        http_body = Some(String::from_utf8_lossy(&body).to_string());

        if method == "POST" {
            return Ok(HTTPRequest::new(
                None,
                method,
                PathBuf::from(path),
                http_version,
                http_headers,
                http_body,
            ));
        }

        Err(HTTPParseError::InvalidStatusLine)
    }
    
    fn handle_connection(stream: TcpStream) -> Result<Option<String>, Option<String>> {
        let buf_reader = BufReader::new(&stream);

        let mut request_object: HTTPRequest = match Self::parse_http_request(buf_reader) {
            Ok(value) => value,
            Err(e) => {
                return Err(Some("Error processing HTTP request: {e}".parse().unwrap()));
            }
        };

        request_object.set_stream(stream);

        // TODO add logging
        resolve_endpoint(request_object)
    }

    pub fn run(&mut self, default: bool, rx: Arc<Mutex<Receiver<()>>>, selected_port: u16) {
        let port: u16;
        if default == true {
            port = Self::DEFAULT_PORT;
        } else {
            port = match selected_port {
                port if (1..=65535).contains(&port) => port,
                _ => {
                    println!("Invalid port! Using default: {}", Self::DEFAULT_PORT);
                    Self::DEFAULT_PORT
                }
            }
        }

        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");

        let thread_pool = match ThreadPool::new(*NUMBER_OF_THREADS_IN_THREAD_POOL) {
            Ok(value) => value,
            Err(e) => panic!("{e}"),
        };

        loop {
            // Check for termination signal
            if let Ok(lock) = rx.try_lock() {
                if let Ok(_) = lock.try_recv() {
                    break;
                }
            };

            // Try accepting a connection
            match listener.accept() {
                Ok((stream, _)) => {
                    let logger = Arc::clone(&self.logger);
                    thread_pool.execute(move || {
                        match Self::handle_connection(stream) {
                            Ok(Some(value)) => logger.log(format!("{value}")),
                            Err(Some(value)) => logger.log_error(format!("{value}")),
                            _ => {}
                        };
                    })
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(Self::REFRESH_RATE_SERVER_IN_MS));
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    break;
                }
            }
        }

        println!("Dropping thread pool");

        drop(thread_pool);
    }
}
