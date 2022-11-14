use clap::{ArgGroup, Args, Parser, Subcommand};
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

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Passively seed files and dht.
    #[clap(name = "seed")]
    Seed,

    /// Test leech
    #[clap(name = "leech")]
    Leech,

    /// Ping a given node by its id
    #[clap(name = "ping")]
    Ping(Target),
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
    println!("Node ID is: {}", node_id);

    match args.command {
        Command::Seed => {
            let addr = args.server_addr.parse()?;
            let node_id = 0; // FIXME temp override
            let mut manager = Manager::new(node_id, addr, "/tmp/seeder".to_owned());
            // FIXME, load_dir
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        Command::Ping(target) => {
            let _own_addr: SocketAddr = args.server_addr.parse()?;
            let own_addr = "127.0.0.1:4001".parse()?; // FIXME
            let mut manager = Manager::new(1, own_addr, "/tmp/leecher".to_owned());
            manager.set_max_hop(args.max_hop);
            if manager.load_dht(Path::new("/tmp/dht")).await.is_err() {
                println!("can't find dht file");
            }
            let succeed = manager.ping(target.id).await?;
            println!("Node {} acknowledge: {}", target.id, succeed);
        }
        Command::Leech => {
            let own_addr = "127.0.0.1:4001".parse()?;
            let peer_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(1, own_addr, "/tmp/leecher".to_owned());
            manager.set_max_hop(args.max_hop);
            if manager.load_dht(Path::new("/tmp/dht")).await.is_err() {
                println!("can't find dht file");
            }
            manager.bootstrap(peer_addr).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
            manager.download_file(3613099103).await?;

            let succeed = manager.send_message(0, "hello dear server".to_owned()).await?;
            println!("Message sent: {}", succeed);
        }
    }

    Ok(())
}
