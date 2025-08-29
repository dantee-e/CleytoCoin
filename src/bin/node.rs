use cleyto_coin::{run_server, run_server_with_gui};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    gui: bool,
}

fn main() {
    let args = Args::from_args();
    match args.gui {
        true => run_server_with_gui().unwrap(),
        false => {
            run_server();
        }
    }
}
