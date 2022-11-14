use clap::{ArgGroup, Args, Parser, Subcommand};
use colored::Colorize;
use errors::AnyResult;
use piretoutpire::manager::manager::Manager;
use rand::Rng;
use std::{net::SocketAddr, path::Path};

#[derive(Parser)]
#[clap(name = "PireToutPire")]
#[clap(about = "CLI for the p2p network", long_about = None)]

struct Cli {
    /// Listening address for receiving commands
    #[clap(default_value_t = String::from("127.0.0.1:4000"))]
    #[clap(long, value_name = "host:port")]
    server_addr: String,

    /// Node id (empty = random)
    #[clap(long, value_name = "node_id")]
    node_id: Option<u32>,

    /// Max hop (empty = default behavior, search until not closer)
    #[clap(long, value_name = "max_hop")]
    max_hop: Option<u32>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Eq, PartialEq)]
pub enum Command {
    /// Passively seed files and dht.
    #[clap(name = "seed")]
    Seed,

    /// Test leech
    #[clap(name = "leech")]
    Leech,

    /// Ping a user from its peer id
    #[clap(arg_required_else_help = true)]
    #[clap(name = "ping")]
    Ping {
        /// Peer id
        #[clap(value_parser)]
        target: u32,
    },

    /// Bootstrap this node by giving a known peer address
    #[clap(arg_required_else_help = true)]
    #[clap(name = "bootstrap")]
    Bootstrap {
        /// Peer address
        #[clap(value_parser)]
        peer_addr: String,
    },

    /// Clones repos
    #[clap(arg_required_else_help = true)]
    #[clap(name = "download")]
    DownloadFile {
        /// Peer id
        #[clap(value_parser)]
        file_crc: u32,
    },

    /// Find the given nodes by its id, or return the 4 closest
    #[clap(arg_required_else_help = true)]
    #[clap(name = "find-node")]
    FindNode {
        /// Peer id
        #[clap(value_parser)]
        target: u32,
    },

    /// Download a file by its crc
    #[clap(arg_required_else_help = true)]
    #[clap(name = "message")]
    Message {
        /// Peer id
        #[clap(value_parser)]
        target: u32,
        /// Message
        #[clap(value_parser)]
        message: String,
    },
    // /// Download a file by its crc
    // #[clap(name = "message")]
    // Message(MessageTarget),
}

#[derive(Debug, Args)]
#[clap(group(
    ArgGroup::new("target")
        .required(true)
        .args(&["id"]),
))]
pub struct Target {
    /// Target every data source.
    #[clap(short, long)]
    pub id: u32,
}

#[derive(Debug, Args)]
pub struct MessageTarget {
    /// Peer id
    #[clap(short, long)]
    pub id: u32,

    /// Message to send
    #[clap(short, long)]
    pub msg: String,
}

// Mode ------------------------------------------------------------------------

// Generate a random node id.
fn get_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args = Cli::parse();
    let node_id = args.node_id.unwrap_or_else(|| get_random_id());
    let own_addr: SocketAddr = args.server_addr.parse()?;

    // HACK
    let (own_addr, node_id) = if args.command == Command::Seed {
        ("127.0.0.1:4000".parse()?, 0)
    } else {
        ("127.0.0.1:4001".parse()?, 1)
    };
    // !HACK

    let info = format!("Node ID is: {}, and own server address is {}", node_id, own_addr);
    println!("{}", info.truecolor(135, 138, 139).italic());

    let mut manager = Manager::new(node_id, own_addr, "/tmp/leecher".to_owned());
    manager.set_max_hop(args.max_hop);
    if manager.load_dht(Path::new("/tmp/dht")).await.is_err() {
        println!("can't find dht file");
    }

    match args.command {
        Command::Seed => {
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        Command::Bootstrap { peer_addr } => {
            manager.bootstrap(peer_addr.parse()?).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
        }
        Command::Ping { target } => {
            let succeed = manager.ping(target).await?;
            println!("Node {} acknowledge: {}", target, succeed);
        }
        Command::FindNode { target } => {
            let nodes = manager.find_node(target).await?;
            println!("Node found are: {:?}", nodes);
        }
        Command::DownloadFile { mut file_crc } => {
            file_crc = 3613099103; // tmp hack
            manager.download_file(file_crc).await?;
            println!("File downloaded: {}", file_crc);
        }
        Command::Message { target, message } => {
            let succeed = manager.send_message(target, message).await?;
            println!("Message send and acknowledge: {}", succeed);
        }
        Command::Leech => {
            manager.bootstrap("127.0.0.1:4000".parse()?).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
            manager.download_file(3613099103).await?;

            let succeed = manager.send_message(0, "hello dear server".to_owned()).await?;
            println!("Message sent: {}", succeed);
        }
    }

    Ok(())
}
