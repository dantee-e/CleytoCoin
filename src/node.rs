mod utils;
mod resolve_requests;
mod thread_pool;

use core::panic;
use std::time::Duration;
use resolve_requests::methods::{self, return_json, HTTPParseError, HTTPRequest, HTTPResponse};
use crate::chain::transaction::Transaction;
use thread_pool::ThreadPool;



use std::{
    collections::HashMap, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::{mpsc::Receiver, Arc, Mutex}, thread
};





pub struct Node {
    port: u16,
    transactions_list: Vec<Transaction>
}

impl Node {
    // these configurations should be moved to a file
    pub const DEFAULT_PORT: u16 = 9473;
    pub const REFRESH_RATE_SERVER_IN_MS: u64 = 50;
    pub const NUMBER_OF_THREADS_IN_THREAD_POOL: usize = 10;

    pub fn new(port: u16) -> Node {
    
        Node {
            port: port,
            transactions_list: Vec::new()
        }
    }

    fn parse_http_request<R: Read>(stream: &TcpStream,mut buf_reader: BufReader<R>) -> Result<HTTPRequest, HTTPParseError> {

        let mut http_headers: HashMap<String, String> = HashMap::new();
        let http_body: Option<String>;


        println!("Beginning of request");

        let mut line = String::new();


        // reading status_line
        let status_line: String;
        
        match buf_reader.read_line(&mut line) {
            Ok(n) if (n > 0) => n,
            Ok(_) => return Err(HTTPParseError::InvalidStatusLine),
            Err(_) => return Err(HTTPParseError::InvalidStatusLine),
        };

        status_line = line.trim().to_string();

        let mut tokens =  status_line.split(' ');
        let (method, path, http_version) = (
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string(),
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string(), 
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string()
        );


        

        
        // reading headers
        loop {
            line.clear();

            if let Err(e) = buf_reader.read_line(&mut line) {
                return Err(HTTPParseError::InvalidRequestLine);
            }

            let line = line.trim_end().to_string();
            
        
            if line.is_empty() {
                break;
            }
            
            if let Some((key, value)) = line.split_once(":") {
                http_headers.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                return_json(stream, HTTPResponse::BadRequest);
            };
        
        }
        
        // If method is GET, return before trying to read the body
        if method == "GET" {
            return Ok(HTTPRequest::new(method, path, http_version, http_headers, None))
        }



        // getting content_lenght from headers
        let content_length = match http_headers.get("content-length") {
            Some(value) => match value.parse::<usize>() {
                Ok(length) => length,
                Err(_) => {
                    return_json(stream, HTTPResponse::BadRequest);
                    return Err(HTTPParseError::MissingContentLength);
                }
            },
            None => {
                return_json(stream, HTTPResponse::BadRequest);
                return Err(HTTPParseError::MissingContentLength);
            },
        };

        // reading body
        let mut body = vec![0; content_length];
        if let Err(e) = buf_reader.read_exact(&mut body) {
            eprintln!("Error reading body: {}", e);
            return_json(&stream, HTTPResponse::BadRequest);
        }

        http_body = Some(String::from_utf8_lossy(&body).to_string());




        



        Ok(HTTPRequest::new(method, path, http_version, http_headers, http_body))
    }

    fn handle_connection(stream: TcpStream){
        let buf_reader = BufReader::new(&stream);
        

        println!("End of request");
        
        let request_object: HTTPRequest;
        match Self::parse_http_request(&stream, buf_reader) {
            Ok(value) => request_object = value,
            Err(e) => {
                println!("Error processing HTTP request: {e}");
                return_json(&stream, HTTPResponse::BadRequest);
                return;
            },
        };

        match request_object.get_method().as_str() {
            "GET" => methods::get(&stream, request_object),
            "POST" => methods::post(&stream, request_object),
            _ => return_json(&stream, HTTPResponse::InvalidMethod),
        } 

        return;
         

    }

    pub fn run(default: bool, rx: Arc<Mutex<Receiver<()>>>, selected_port: u16) {
        let port: u16;
        if default == true {
            port = Self::DEFAULT_PORT; 
        }
        
        else {
            port = match selected_port {
                port if (1..=65535).contains(&port) => port,
                _ => {
                    println!("Invalid port! Using default: {}", Self::DEFAULT_PORT);
                    Self::DEFAULT_PORT
                }
            }
        }
        

        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        let thread_pool = match ThreadPool::new(Self::NUMBER_OF_THREADS_IN_THREAD_POOL){
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
                Ok((stream, addr)) => {
                    thread_pool.execute(move ||{
                        Self::handle_connection(stream);
                    });
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(Self::REFRESH_RATE_SERVER_IN_MS));
                },
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    break;
                }
            }
        }
    }
}