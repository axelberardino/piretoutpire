use super::context::Context;
use crate::{
    manager::server::{serve_file_chunk, serve_file_info, serve_find_node, serve_ping},
    network::protocol::Command,
    read_all,
};
use errors::AnyResult;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{tcp::WriteHalf, TcpStream},
    sync::Mutex,
};

// Command handler -------------------------------------------------------------

// Interpret a command and act accordingly. This is where request/response are
// handled.
async fn dispatch(
    ctx: Arc<Mutex<Context>>,
    sender_addr: SocketAddr,
    writer: &mut BufWriter<WriteHalf<'_>>,
    request: Command,
) -> AnyResult<()> {
    let res_command = match request {
        // Server message handling
        Command::FileInfoRequest(crc) => serve_file_info(ctx, crc).await,
        Command::ChunkRequest(crc, chunk_id) => serve_file_chunk(ctx, crc, chunk_id).await,
        Command::FindNodeRequest(sender, target) => serve_find_node(ctx, sender_addr, sender, target).await,
        Command::PingRequest(crc) => serve_ping(ctx, crc).await,
        Command::StoreRequest(key, message) => todo!(),
        Command::FindValueRequest(key) => todo!(),
        Command::MessageRequest(key, message) => todo!(),

        // Client message handling, shouldn't be reach.
        Command::ChunkResponse(_, _, _)
        | Command::FileInfoResponse(_)
        | Command::FindNodeResponse(_)
        | Command::PingResponse(_)
        | Command::ErrorOccured(_)
        | Command::StoreResponse()
        | Command::FindValueResponse(_)
        | Command::MessageResponse() => unreachable!(),
    };

    let response: Vec<u8> = res_command.into();
    eprintln!("sending buf {:?}", &response);
    writer.write_all(response.as_slice()).await?;
    writer.flush().await?;
    Ok(())
}

// Main handler ----------------------------------------------------------------

// Start to listen to command. One instance will be spawn for each peer.
pub async fn listen_to_command(ctx: Arc<Mutex<Context>>, mut stream: TcpStream) -> AnyResult<()> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("{} is connected", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    loop {
        let raw_order = read_all!(reader);
        let len = raw_order.len();
        if len == 0 {
            break;
        }

        match raw_order.as_slice().try_into() {
            Ok(command) => dispatch(Arc::clone(&ctx), peer_addr, &mut writer, command).await?,
            Err(err) => eprintln!("Unknown command received! {}", err),
        }
    }

    Ok(())
}
