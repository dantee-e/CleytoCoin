use cleyto_coin::{
    add_name_to_running_servers, kill_all_nodes, kill_node, new_server_name, run_server,
    run_server_new_process, run_server_with_gui,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cleyto-coin-wallet")]
enum Args {
    /// Kills the running server
    Kill {
        /// Name of the server to be killed
        node: Option<String>,

        /// Kills all servers
        #[structopt(long, conflicts_with = "node")]
        all: bool,
    },

    /// Start the server. The flag --gui defines if headless or not
    Start {
        #[structopt(long)]
        gui: bool,

        #[structopt(long)]
        blocking: bool,

        #[structopt(long)]
        name: Option<String>,
    },
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Kill { node, all } => {
            if all {
                kill_all_nodes();
            } else {
                let node = node.expect("Provide a node name or use the --all flag");
                kill_node(node).expect("Couldn't kill node");
            }
        }
        Args::Start {
            gui,
            blocking,
            name,
        } => {
            let server_name = if let Some(name) = name {
                name
            } else {
                new_server_name()
            };
            println!("Starting server: {}", server_name);
            add_name_to_running_servers(server_name.clone());

            if gui {
                run_server_with_gui(server_name.clone()).unwrap();
            } else if blocking {
                run_server(server_name.clone());
            } else {
                run_server_new_process(server_name.clone());
            }
        }
    }
}
