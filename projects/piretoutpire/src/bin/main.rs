use clap::{Parser, Subcommand};
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

    /// Peer id (empty = random)
    #[clap(long, value_name = "id")]
    peer_id: Option<u32>,

    /// Max hop (empty = default behavior, search until not closer)
    #[clap(long, value_name = "nb")]
    max_hop: Option<u32>,

    /// Force this peer to wait X ms before answering each rpc (for debug purpose)
    #[clap(long, value_name = "ms")]
    slowness: Option<u32>,

    /// Disable the recent peers cache. On small network, with non uniform id
    /// distribution, caching peers could be hard. The "recent" peers cache is
    /// used on top of the routing table, to help finding peers. On big network,
    /// it's usually not needed and could be disactivated.
    #[clap(long, value_name = "disable-recent-peers-cache", action)]
    disable_recent_peers_cache: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Eq, PartialEq)]
pub enum Command {
    // HACK
    /// Test leech
    #[clap(name = "leech")]
    Leech,

    /// Passively seed files and dht
    #[clap(name = "seed")]
    Seed,

    /// Ping a user from its peer id
    #[clap(arg_required_else_help = true)]
    #[clap(name = "ping")]
    Ping {
        /// Peer id
        #[clap(value_parser)]
        target: u32,
    },

    /// Bootstrap this peer by giving a known peer address. Use it with a big
    /// max-hop to discover peers on a small network
    #[clap(arg_required_else_help = true)]
    #[clap(name = "bootstrap")]
    Bootstrap {
        /// Peer address
        #[clap(value_parser)]
        peer_addr: String,
    },

    /// Download a file, given its crc
    #[clap(arg_required_else_help = true)]
    #[clap(name = "download")]
    DownloadFile {
        /// Peer id
        #[clap(value_parser)]
        file_crc: u32,
    },

    /// Ask a peer for file description, given its crc
    #[clap(arg_required_else_help = true)]
    #[clap(name = "file-info")]
    FileInfo {
        /// Peer id
        #[clap(value_parser)]
        file_crc: u32,
    },

    /// Find the given peer by its id, or return the 4 closest
    #[clap(arg_required_else_help = true)]
    #[clap(name = "find-node")]
    FindNode {
        /// Peer id
        #[clap(value_parser)]
        target: u32,
    },

    /// Store a value on the dht.
    #[clap(arg_required_else_help = true)]
    #[clap(name = "store")]
    Store {
        /// Key
        #[clap(value_parser)]
        key: u32,
        /// value
        #[clap(value_parser)]
        value: String,
    },

    /// Store a value on the dht.
    #[clap(arg_required_else_help = true)]
    #[clap(name = "find-value")]
    FindValue {
        /// Key
        #[clap(value_parser)]
        key: u32,
    },

    /// Send a message to a given peer
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
}

// Mode ------------------------------------------------------------------------

// Generate a random peer id.
fn get_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args = Cli::parse();
    let peer_id = args.peer_id.unwrap_or_else(|| get_random_id());
    let own_addr: SocketAddr = args.server_addr.parse()?;

    // HACK
    let (own_addr, peer_id) = if args.command == Command::Seed {
        ("127.0.0.1:4000".parse()?, 0)
    } else {
        ("127.0.0.1:4001".parse()?, 1)
    };
    // !HACK

    let info = format!("Peer ID is: {}, and own server address is {}", peer_id, own_addr);
    println!("{}", info.truecolor(135, 138, 139).italic());

    let mut manager = Manager::new(peer_id, own_addr, "/tmp/leecher".to_owned());
    manager.set_max_hop(args.max_hop);
    manager
        .set_recent_peers_cache_enable(!args.disable_recent_peers_cache)
        .await;
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
            let peers = manager.find_node(target).await?;
            println!("Node found are: {:?}", peers);
        }
        Command::DownloadFile { mut file_crc } => {
            file_crc = 3613099103; // tmp hack
            manager.download_file(file_crc).await?;
            println!("File downloaded: {}", file_crc);
        }
        Command::FileInfo { mut file_crc } => {
            file_crc = 3613099103; // tmp hack
            let file_info = manager.file_info(file_crc).await?;
            println!("File info: {:?}", file_info);
        }
        Command::Store { key, value } => {
            manager.store_value(key, value).await?;
            println!("Value has been stored");
        }
        Command::FindValue { key } => {
            let value = manager.find_value(key).await?;
            println!("Value found is: {:?}", value);
        }
        Command::Message { target, message } => {
            let succeed = manager.send_message(target, message).await?;
            println!("Message send and acknowledge: {}", succeed);
        }
        Command::Leech => {
            manager.bootstrap("127.0.0.1:4000".parse()?).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
            let succeed = manager.send_message(0, "hello dear server".to_owned()).await?;
            println!("Message sent: {}", succeed);

            manager.download_file(3613099103).await?;
        }
    }

    Ok(())
}
