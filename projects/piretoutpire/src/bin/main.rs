use core::time;
use errors::{bail, AnyError, AnyResult};
use piretoutpire::file::file_chunk::FileChunk;
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
}

impl TryFrom<&str> for ConnectionType {
    type Error = AnyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "client" => Self::Client,
            "server" => Self::Server,
            _ => bail!("unknown type {}", value),
        })
    }
}

fn main() -> AnyResult<()> {
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
    let fc = FileChunk::open_existing(&file);

    match connection_type {
        ConnectionType::Client => {
            let mut stream = TcpStream::connect(addr)?;
            for i in 0..10 {
                thread::sleep(time::Duration::from_millis(100));
                stream.write_all(&[10 + i, 20 + i, 30 + i, 40 + i])?;
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
    }

    Ok(())
}
