pub mod chain;
pub mod node;

use std::{sync::{mpsc, Arc, Mutex}, thread};
use std::io::Read;
fn main(){
    let (_, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        node::Node::run(true, rx, 0);
        
    });


    let mut input = String::new();

    loop {
        print!("command: ");
        
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        input = input.trim().to_string();
        println!("{:?}", input);
        
        match input.as_str() {
            "quit" => {println!("quit program"); break;},
            _ => println!("no command")

        }
        

    }

    
}