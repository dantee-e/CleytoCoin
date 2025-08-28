use cleyto_coin::chain::Chain;
use cleyto_coin::node::{self, ui::App};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    gui: bool,
}

fn run_with_gui() -> color_eyre::Result<()> {
    let (tx, rx) = mpsc::channel::<()>();

    // Channel to kill thread
    let rx = Arc::new(Mutex::new(rx));

    let (mut node, logger) = node::Node::new(Chain::new());

    // Run server thread
    let server = thread::spawn(move || {
        let rx = Arc::clone(&rx);

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
fn run_server() {
    let (_, rx) = mpsc::channel::<()>();
    let rx = Arc::new(Mutex::new(rx));
    let (mut node, _) = node::Node::new(Chain::new());
    node.run(true, rx, 0);
}

fn main() {
    let args = Args::from_args();
    match args.gui {
        true => run_with_gui().unwrap(),
        false => run_server(),
    }
    run_with_gui().unwrap();
}
