use cleyto_coin::{kill_server, run_server, run_server_new_process, run_server_with_gui};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cleyto-coin-wallet")]
enum Args {
    /// Kills the running server
    Kill {},

    /// Start the server. The flag --gui defines if headless or not
    Start {
        #[structopt(long)]
        gui: bool,

        #[structopt(long = "blocking")]
        blocking: bool,
    },
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Kill {} => kill_server(),
        Args::Start { gui, blocking } => match gui {
            true => run_server_with_gui().unwrap(),
            false => {
                if blocking {
                    run_server();
                } else {
                    run_server_new_process();
                }
            }
        },
    }
}
