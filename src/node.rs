mod utils;
mod resolve_requests;

use resolve_requests::methods::{self, HTTPParseError};
use crate::chain::transaction::Transaction;



use std::{
    collections::HashMap, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};
use std::io;





pub struct Node {
    port: u16,
    transactions_list: Vec<Transaction>
}

impl Node {
    const DEFAULT_PORT: u16 = 9473;

    pub fn new(port: u16) -> Self {
        Node {
            port: port,
            transactions_list: Vec::new()
        }
    }

    fn parse_http_request(http_request: Vec<String>) -> Result<methods::HTTPRequest, methods::HTTPParseError> {
        let mut tokens =  http_request[0].split(' ');
        let (method, path, http_version) = (
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string(),
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string(), 
            tokens.next().ok_or(HTTPParseError::InvalidRequestLine)?.to_string()
        );

        let headers = HashMap::new();
        let body = Some(String::new());




        Ok(methods::HTTPRequest::new(method, path, http_version, headers, body))
    }

    fn handle_connection(stream: TcpStream){
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut request_tokens = http_request[0].split(' '); // iterator
        
        let request_object = Self::parse_http_request(http_request);
        

    }

    pub fn run() {
        let port: u16;
        let mut input = String::new();

        println!("Please input the port you want to use (Press enter for default {}): ", Self::DEFAULT_PORT);
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read port");


        port = if input.trim().is_empty() {
            Self::DEFAULT_PORT
        } else {
            match input.trim().parse::<u16>() {
                Ok(port) if (1..=65535).contains(&port) => port,
                _ => {
                    println!("Invalid port! Using default: {}", Self::DEFAULT_PORT);
                    Self::DEFAULT_PORT
                }
            }
        };

        println!("Activating listener...");

        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        println!("Listening to new connections on port {port}!");


        for stream in listener.incoming() {
            let stream = stream.unwrap();
            Self::handle_connection(stream);

            println!("Connection established!");
        }
    }
}