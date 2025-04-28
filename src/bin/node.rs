use std::{sync::{mpsc, Arc, Mutex}, thread};
use cleyto_coin::chain::Chain;
use cleyto_coin::node::{
    self,
    ui::App,
    logger::Logger
};



fn main() -> color_eyre::Result<()> {

    let logger = Arc::new(Logger::new());

    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));
    
    
    // Run server thread
    let logger_clone_for_node = Arc::clone(&logger);
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);
        let mut node = node::Node::new(Chain::new(), logger_clone_for_node);
        node.run(true, rx, 0);
    });


    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new(Arc::clone(&logger), node::Node::DEFAULT_PORT).run(terminal);
    ratatui::restore();

    // Quits server
    tx.send(())?;

    server.join().unwrap();
    result
}