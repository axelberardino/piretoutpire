use errors::{bail, AnyError, AnyResult};
use piretoutpire::manager::manager::Manager;
use std::path::Path;

enum ConnectionType {
    Seeder,
    Leecher,
}

impl TryFrom<&str> for ConnectionType {
    type Error = AnyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "seeder" => Self::Seeder,
            "leecher" => Self::Leecher,
            _ => bail!("unknown type {}", value),
        })
    }
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("missing action (client or server)");
    }
    if args.len() <= 2 {
        panic!("missing host:port");
    }
    // if args.len() <= 3 {
    //     panic!("missing working folder");
    // }
    println!("{:?}", args);

    let connection_type: ConnectionType = args[1].as_str().try_into()?;
    let _addr = args[2].clone();
    let _file = if !args[3].is_empty() {
        args[3].clone()
    } else {
        "/tmp/toto.txt".to_owned()
    };
    match connection_type {
        ConnectionType::Seeder => {
            let server_own_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(0, server_own_addr, "/tmp/seeder".to_owned());
            manager.load_file("/tmp/seeder/toto.txt").await?;
            manager.start_server().await?;
        }
        ConnectionType::Leecher => {
            let own_addr = "127.0.0.1:4001".parse()?;
            let peer_addr = "127.0.0.1:4000".parse()?;
            let mut manager = Manager::new(1, own_addr, "/tmp/leecher".to_owned());
            if manager.load_dht(Path::new("/tmp/dht")).await.is_err() {
                eprintln!("can't find dht file");
            }
            manager.bootstrap(peer_addr).await?;
            manager.dump_dht(Path::new("/tmp/dht")).await?;
            manager.download_file(3613099103).await?;
        }
    }

    Ok(())
}
