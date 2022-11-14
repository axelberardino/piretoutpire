use clap::{ArgEnum, Parser};
use errors::AnyResult;
use piretoutpire::manager::manager::Manager;
use std::path::Path;

#[derive(Parser)]
#[clap(name = "pire2pire")]
#[clap(about = "Pire2Pire CLI", long_about = None)]

struct Cli {
    /// Server host:port
    #[clap(default_value_t = String::from("localhost:4000"))]
    #[clap(long, value_name = "host:port")]
    server: String,

    /// What mode to run the program in
    #[clap(arg_enum)]
    #[clap(long = "type", value_name = "type")]
    query_type: QueryType,
}

// Mode ------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum QueryType {
    Seeder,
    Leecher,
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args = Cli::parse();

    match &args.query_type {
        QueryType::Seeder => {
            let server_own_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(0, server_own_addr, "/tmp/seeder".to_owned());
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        QueryType::Leecher => {
            let own_addr = "127.0.0.1:4001".parse()?;
            let peer_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(1, own_addr, "/tmp/leecher".to_owned());
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
