use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use std::io;

pub struct Node {
    default_port: u16
}

impl Node {

    const DEFAULT_PORT: u16 = 9473;

    pub fn new(port: u16) -> Self {
        Node {
            default_port: port
        }
    }

    fn handle_connection(stream: TcpStream){
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!("Request: {http_request:#?}");
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

        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            Self::handle_connection(stream);

            println!("Connection established!");
        }
    }
}