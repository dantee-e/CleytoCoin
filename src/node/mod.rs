pub mod logger;
mod resolve_requests;
mod thread_pool;
pub mod ui;
mod utils;
use crate::chain::{transaction::Transaction, Chain};
use crate::configs::KILL_SERVER_SOCKET_PATH;
use crate::node::logger::Logger;
use core::panic;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use resolve_requests::endpoints::resolve_endpoint;
use resolve_requests::methods::{HTTPParseError, HTTPRequest};
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use thread_pool::custom_thread_pool::ThreadPool;

#[derive(Serialize, Deserialize)]
pub struct NodeState {
    status: bool,
    chain: Chain,
    transactions_pool: Vec<Transaction>,
}
#[derive(Debug, Serialize, Deserialize)]
struct NodeConfig {
    log_path: PathBuf,
    socket_path: PathBuf,
}
impl Default for NodeConfig {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("", "CleytoCoin Big Mean Corp", "cleyto_coin")
            .expect("Could not find the config directory");
        Self {
            log_path: proj_dirs.data_dir().join("logs.log"),
            socket_path: PathBuf::from(KILL_SERVER_SOCKET_PATH),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    state: Arc<Mutex<NodeState>>,
    #[serde(skip)] // The logs are manually saved on shutdown and reloaded on initialization
    logger: Arc<logger::Logger>,
    #[serde(skip)] // The configs are best reloaded with every initialization
    config: NodeConfig,
}

static NUMBER_OF_THREADS_IN_THREAD_POOL: Lazy<usize> = Lazy::new(num_cpus::get);

// 0 = None
// 1 = Prod
// 2 = Debug
pub const LOG_LEVEL: u8 = 2;

fn load_config() -> NodeConfig {
    let proj_dirs = ProjectDirs::from("", "CleytoCoin Big Mean Corp", "cleyto_coin")
        .expect("Could not find the config directory");
    let config_path = proj_dirs.config_dir().join("config.toml");

    if let Ok(contents) = fs::read_to_string(&config_path) {
        toml::from_str(&contents).expect("Invalid config format")
    } else {
        fs::create_dir_all(proj_dirs.config_dir()).expect("Could not create config directories");

        let default_cfg = NodeConfig::default();
        let toml_str = toml::to_string_pretty(&default_cfg).unwrap();
        fs::write(&config_path, &toml_str).expect("Couldn't write default config");
        default_cfg
    }
}

impl Node {
    // these configurations should be moved to a file
    pub const DEFAULT_PORT: u16 = 9473;
    pub const REFRESH_RATE_SERVER_IN_MS: u64 = 50;

    pub fn new(chain: Chain) -> (Node, Arc<Logger>) {
        let config = load_config();
        let logger =
            Arc::new(Logger::read_logs_file(&config.log_path).unwrap_or_else(|_| Logger::new()));
        let logger_clone = Arc::clone(&logger);
        (
            Node {
                state: Arc::new(Mutex::new(NodeState {
                    status: true,
                    chain,
                    transactions_pool: Vec::new(),
                })),
                logger,
                config,
            },
            logger_clone,
        )
    }

    fn parse_http_request<R: Read>(
        mut buf_reader: BufReader<R>,
    ) -> Result<HTTPRequest, HTTPParseError> {
        let mut http_headers: HashMap<String, String> = HashMap::new();

        let mut line = String::new();

        // reading status_line

        match buf_reader.read_line(&mut line) {
            Ok(n) if (n > 0) => n,
            Ok(_) => return Err(HTTPParseError::InvalidStatusLine),
            Err(_) => return Err(HTTPParseError::InvalidStatusLine),
        };

        let status_line: String = line.trim().to_string();

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

            if buf_reader.read_line(&mut line).is_err() {
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

        let http_body: Option<String> = Some(String::from_utf8_lossy(&body).to_string());

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

    fn handle_connection(
        state: Arc<Mutex<NodeState>>,
        stream: TcpStream,
    ) -> Result<Option<String>, Option<String>> {
        let buf_reader = BufReader::new(&stream);

        let mut request_object: HTTPRequest = match Self::parse_http_request(buf_reader) {
            Ok(value) => value,
            Err(e) => {
                return Err(Some(format!("Error processing HTTP request: {e}")));
            }
        };

        request_object.set_stream(stream);

        // TODO add logging
        resolve_endpoint(state, request_object)
    }

    pub fn run(&mut self, default: bool, selected_port: u16) {
        let port: u16 = if default {
            Self::DEFAULT_PORT
        } else {
            match selected_port {
                port if (1..=65535).contains(&port) => port,
                _ => {
                    println!("Invalid port! Using default: {}", Self::DEFAULT_PORT);
                    Self::DEFAULT_PORT
                }
            }
        };

        let tcp_listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        tcp_listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");

        let thread_pool = match ThreadPool::new(*NUMBER_OF_THREADS_IN_THREAD_POOL) {
            Ok(value) => value,
            Err(e) => panic!("{e}"),
        };

        // The termination signal will be a socket now
        let parent = self.config.socket_path.parent().unwrap();
        std::fs::create_dir_all(parent).expect("Could not create temp dirs for parent socket");

        let mut read_buffer: [u8; 100] = [0u8; 100];
        loop {
            if let Ok(mut listener) = UnixStream::connect(KILL_SERVER_SOCKET_PATH) {
                let command: Option<&str> = match listener.read(&mut read_buffer) {
                    Ok(n) => str::from_utf8(&read_buffer[..n]).ok(),
                    Err(_) => None,
                };

                match command {
                    Some("kill") => break,
                    Some(&_) => {}
                    None => {}
                }
            }
            // Check for local signal

            // Try accepting a connection
            match tcp_listener.accept() {
                Ok((stream, _)) => {
                    let logger = Arc::clone(&self.logger);
                    let state = Arc::clone(&self.state);
                    thread_pool.execute(move || {
                        match Self::handle_connection(state, stream) {
                            Ok(Some(value)) => {
                                logger.log(value);
                            }
                            Err(Some(value)) => logger.log_error(value),
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
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        match self.logger.write_logs_file(&self.config.log_path) {
            Ok(_) => {}
            Err(e) => eprintln!("Error saving log file: {e:?}"),
        }
    }
}
