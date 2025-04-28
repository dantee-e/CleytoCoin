use std::{sync::{mpsc, Arc, Mutex}, thread};
use cleyto_coin::node::{self, ui::App};




fn main() -> color_eyre::Result<()> {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        node::Node::run(true, rx, 0);

    });


    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal, node::Node::DEFAULT_PORT);
    ratatui::restore();
    
    // Quits server
    tx.send(())?;
    
    server.join().unwrap();
    result
}