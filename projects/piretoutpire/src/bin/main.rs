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
    /// Listening address for receiving commands.
    #[clap(default_value_t = String::from("127.0.0.1:4000"))]
    #[clap(long, value_name = "host:port")]
    server_addr: String,

    /// Peer id (empty = random).
    #[clap(long, value_name = "id")]
    peer_id: Option<u32>,

    /// Max hop (empty = default behavior, search until not closer).
    /// Setting this option will enable a more greedy strategy for peers
    /// finding.
    #[clap(long, value_name = "nb")]
    max_hop: Option<u32>,

    /// Force this peer to wait X ms before answering each rpc (for debug
    /// purpose).
    #[clap(long, value_name = "ms")]
    slowness: Option<u64>,

    /// Max wait time for initiating a connection (default is 200 ms).
    #[clap(long, value_name = "ms")]
    connection_timeout: Option<u64>,

    /// Max wait time for sending a query (default is 200 ms).
    #[clap(long, value_name = "ms")]
    write_timeout: Option<u64>,

    /// Max wait time for receiving a query (default is 200 ms).
    #[clap(long, value_name = "ms")]
    read_timeout: Option<u64>,

    /// Frequency at which the dht is dump into the disk (default is 30 sec).
    #[clap(long, value_name = "ms")]
    dht_dump_frequency: Option<u64>,

    /// Config file for dht.
    #[clap(default_value_t = String::from("/tmp/dht"))]
    #[clap(long, value_name = "dht-filename")]
    dht_filename: String,

    /// Where the downloaded files and the ones to seed are located.
    #[clap(default_value_t = String::from("/tmp/"))]
    #[clap(long, value_name = "working-dir")]
    working_directory: String,

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

    /// Send to closest node the crc of the file we're sharing
    #[clap(arg_required_else_help = true)]
    #[clap(name = "announce")]
    Announce {
        /// Crc of the file
        #[clap(value_parser)]
        crc: u32,
    },

    /// Get the peers who are owning the wanted file
    #[clap(arg_required_else_help = true)]
    #[clap(name = "get-peers")]
    GetPeers {
        /// Crc of the file
        #[clap(value_parser)]
        crc: u32,
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
    let mut args = Cli::parse();
    let peer_id = args.peer_id.unwrap_or_else(|| get_random_id());
    let own_addr: SocketAddr = args.server_addr.parse()?;

    // HACK
    // file_crc = 3613099103
    let (own_addr, peer_id) = if args.command == Command::Seed {
        args.dht_filename = "/tmp/dht_server".to_owned();
        ("127.0.0.1:4000".parse()?, 0)
    } else {
        ("127.0.0.1:4001".parse()?, 1)
    };
    // !HACK

    let mut manager = Manager::new(
        peer_id,
        own_addr,
        args.dht_filename.clone(),
        args.working_directory,
    );
    manager.set_max_hop(args.max_hop);
    manager
        .set_recent_peers_cache_enable(!args.disable_recent_peers_cache)
        .await;
    manager.set_slowness(args.slowness).await;
    manager.set_connection_timeout(args.connection_timeout).await;
    manager.set_write_timeout(args.write_timeout).await;
    manager.set_read_timeout(args.read_timeout).await;
    manager.set_dht_dump_frequency(args.dht_dump_frequency).await;

    if manager.load_dht(Path::new(&args.dht_filename)).await.is_err() {
        println!(
            "No dht file yet at {}, a new one will be created...",
            &args.dht_filename
        );
    }

    let info = format!(
        "Peer ID: {}, Peer address: {}, Known peers: {}",
        peer_id,
        own_addr,
        manager.known_peers_count().await
    );
    println!("{}", info.truecolor(135, 138, 139).italic());

    match args.command {
        Command::Seed => {
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        Command::Bootstrap { peer_addr } => {
            let peer_found = manager.bootstrap(peer_addr.parse()?).await?;
            manager.dump_dht().await?;
            println!("Bootstrap done, find node: {:?}", peer_found);
        }
        Command::Ping { target } => {
            let succeed = manager.ping(target).await?;
            println!("Node {} acknowledge: {}", target, succeed);
        }
        Command::FindNode { target } => {
            let peers = manager.find_node(target).await?;
            println!("Node found are: {:?}", peers);
        }
        Command::DownloadFile { file_crc } => {
            manager.download_file(file_crc).await?;
            println!("File downloaded: {}", file_crc);
        }
        Command::FileInfo { file_crc } => {
            let file_info = manager.file_info(file_crc).await?;
            println!("File info: {:?}", file_info);
        }
        Command::Store { key, value } => {
            let nb_ack = manager.store_value(key, value).await?;
            println!("{} peers sotre this value", nb_ack);
        }
        Command::FindValue { key } => {
            let value = manager.find_value(key).await?;
            println!("Value found is: {:?}", value);
        }
        Command::Message { target, message } => {
            let succeed = manager.send_message(target, message).await?;
            println!("Message send and acknowledge: {}", succeed);
        }
        Command::Announce { crc } => {
            let nb_ack = manager.announce(crc).await?;
            println!("{} peers acknowledged we're sharing this file", nb_ack);
        }
        Command::GetPeers { crc } => {
            let peers = manager.get_peers(crc).await?;
            println!("Peers who own this file are: {:?}", peers);
        }
        Command::Leech => {
            let peer_found = manager.bootstrap("127.0.0.1:4000".parse()?).await?;
            manager.dump_dht().await?;
            println!("Bootstrap done, find node: {:?}", peer_found);

            let succeed = manager.send_message(0, "hello dear server".to_owned()).await?;
            println!("Message sent: {}", succeed);

            let res = manager.download_file(2021542958).await?;
            println!("download: {:?}", res);
        }
    }

    Ok(())
}
