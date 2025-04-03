mod utils;
mod resolve_requests;

use resolve_requests::methods;
use crate::chain::transaction::Transaction;



use std::{
    io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
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

    

    fn handle_connection(stream: TcpStream){
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut request_tokens = http_request[0].split(' '); // iterator
        
        let method = if let Some(token) = request_tokens.next() {
            token
        } else {
            methods::return_json(&stream, methods::HTTPResponse::BadRequest);
            "shit happened"
        };

        let endpoint = if let Some(token) = request_tokens.next() {
            token
        } else {
            methods::return_json(&stream, methods::HTTPResponse::BadRequest);
            "shit happened"
        };

        match method {
            "GET" => methods::get(&stream, endpoint),
            "POST" => methods::post(&stream, endpoint),
            _ => {
                methods::return_json(&stream, methods::HTTPResponse::InvalidMethod);
            }
        }

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