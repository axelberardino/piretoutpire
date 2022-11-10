use core::time;
use errors::{bail, AnyError, AnyResult};
use piretoutpire::{file::file_chunk::FileChunk, manager::manager::Manager};
use std::{
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let peer_addr = stream.peer_addr().expect("Stream has peer_addr");
    eprintln!("Incoming from {}", peer_addr);
    let mut reader = BufReader::new(stream.try_clone()?);
    // let mut writer = BufWriter::new(stream);
    let mut chunk: Vec<u8> = Vec::new();

    loop {
        // Get data into receiver!
        let received = reader.fill_buf()?;
        let len = received.len();
        if len == 0 {
            break;
        }
        println!("{:?}", &received);
        chunk.extend(received);

        // Advance cursor.
        reader.consume(len);

        // writer.write(received.as_slice())?;
        // writer.flush()
    }

    println!("Received {:?}", chunk);
    Ok(())
}

enum ConnectionType {
    Client,
    Server,
    Seeder,
    Leecher,
}

impl TryFrom<&str> for ConnectionType {
    type Error = AnyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "client" => Self::Client,
            "server" => Self::Server,
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
    let addr = args[2].clone();
    let file = if !args[3].is_empty() {
        args[3].clone()
    } else {
        "/tmp/toto.txt".to_owned()
    };
    match connection_type {
        ConnectionType::Client => {
            let mut fc = FileChunk::open_existing(&file)?;
            let mut stream = TcpStream::connect(addr)?;
            for chunk_id in 0..fc.nb_chunks() {
                thread::sleep(time::Duration::from_millis(100));
                let raw_chunk = fc.read_chunk(chunk_id)?;
                stream.write_all(&raw_chunk)?;
                stream.flush()?;
            }
        }
        ConnectionType::Server => {
            eprintln!("Starting server on '{}'", addr);

            let listener = TcpListener::bind(addr)?;
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    std::thread::spawn(move || {
                        handle_connection(stream).map_err(|e| eprintln!("Error: {}", e))
                    });
                } else {
                    println!("stream fail")
                }
            }
        }
        ConnectionType::Seeder => {
            let mut manager = Manager::new("127.0.0.1:4000".parse()?);
            manager.share_existing_file("/tmp/toto.txt").await?;
        }
        ConnectionType::Leecher => {
            let mut manager = Manager::new("127.0.0.1:4001".parse()?);
            manager.download_file(0).await?;
        }
    }

    Ok(())
}
