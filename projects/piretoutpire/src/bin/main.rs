use clap::{ArgEnum, Parser};
use errors::AnyResult;
use piretoutpire::manager::manager::Manager;
use rand::Rng;
use std::path::Path;

#[derive(Parser)]
#[clap(name = "pire2pire")]
#[clap(about = "Pire2Pire CLI", long_about = None)]

struct Cli {
    /// Server host:port
    #[clap(default_value_t = String::from("127.0.0.1:4000"))]
    #[clap(long, value_name = "host:port")]
    server_addr: String,

    /// Node id (empty = random)
    #[clap(long, value_name = "node_id")]
    node_id: Option<u32>,

    /// Max hop (empty = default behavior, search until not closer)
    #[clap(long, value_name = "max_hop")]
    max_hop: Option<u32>,

    /// What mode to run the program in
    #[clap(arg_enum)]
    #[clap(long = "type", value_name = "type")]
    query_type: QueryType,
}

// Mode ------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum QueryType {
    Seed,
    Leech,
}

// Generate a random node id.
fn get_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args = Cli::parse();
    let node_id = args.node_id.unwrap_or_else(|| get_random_id());
    eprintln!("Node ID is: {}", node_id);

    match &args.query_type {
        QueryType::Seed => {
            let addr = args.server_addr.parse()?;
            let node_id = 0; // FIXME temp override
            let mut manager = Manager::new(node_id, addr, "/tmp/seeder".to_owned());
            // FIXME, load_dir
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        QueryType::Leech => {
            let own_addr = "127.0.0.1:4001".parse()?;
            let peer_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(1, own_addr, "/tmp/leecher".to_owned());
            manager.set_max_hop(args.max_hop);
            if manager.load_dht(Path::new("/tmp/dht")).await.is_err() {
                eprintln!("can't find dht file");
            }
            manager.bootstrap(peer_addr).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
            manager.download_file(3613099103).await?;

            let succeed = manager.send_message(0, "hello dear server".to_owned()).await?;
            eprintln!("Message sent: {}", succeed);
        }
    }

    Ok(())
}
