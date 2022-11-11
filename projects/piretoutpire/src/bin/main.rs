use errors::{bail, AnyError, AnyResult};
use piretoutpire::manager::manager::Manager;

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
            let mut manager = Manager::new("127.0.0.1:4000".parse()?, "/tmp/seeder".to_owned());
            manager.share_existing_file("/tmp/seeder/toto.txt").await?;
        }
        ConnectionType::Leecher => {
            let mut manager = Manager::new("127.0.0.1:4001".parse()?, "/tmp/leecher".to_owned());
            manager.download_file(3613099103).await?;
        }
    }

    Ok(())
}
