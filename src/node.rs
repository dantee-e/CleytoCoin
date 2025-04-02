use std::net::TcpListener;
use std::io;

struct Node {
    let default_port: str = "9473";
}

impl Node {
    pub fn run() {
        let mut port = String::new();
        

        println!("Please input the port you want to use (Press enter for default {default_port}): ");
        io::stdin()
            .read_line(&mut port)
            .expect("Failed to read port");

        let listener = TcpListener::bind("127.0.0.1:" + default_port).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            println!("Connection established!");
        }
    }
}