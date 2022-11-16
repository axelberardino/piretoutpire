use super::context::Context;
use crate::{
    manager::server::{
        serve_announce, serve_file_chunk, serve_file_info, serve_find_node, serve_find_value,
        serve_get_peers, serve_message, serve_ping, serve_store,
    },
    network::protocol::Command,
    read_all,
};
use errors::AnyResult;
use std::{
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{tcp::WriteHalf, TcpStream},
    sync::Mutex,
    time::{sleep, timeout},
};

// Command handler -------------------------------------------------------------

// Interpret a command and act accordingly. This is where request/response are
// handled.
async fn dispatch(
    main_ctx: Arc<Mutex<Context>>,
    incoming_addr: SocketAddr,
    writer: &mut BufWriter<WriteHalf<'_>>,
    request: Command,
    own_id: u32,
) -> AnyResult<()> {
    let ctx = Arc::clone(&main_ctx);
    let (sender, res_command) = match request {
        // Server message handling
        Command::FileInfoRequest(crc) => (None, serve_file_info(ctx, incoming_addr, crc).await),
        Command::ChunkRequest(crc, chunk_id) => {
            (None, serve_file_chunk(ctx, incoming_addr, crc, chunk_id).await)
        }
        Command::FindNodeRequest(peer_sender, target) => (
            Some(peer_sender.id),
            serve_find_node(ctx, peer_sender.addr, peer_sender.id, target).await,
        ),
        Command::PingRequest(peer_sender) => (
            Some(peer_sender.id),
            serve_ping(ctx, peer_sender.addr, peer_sender.id, own_id).await,
        ),
        Command::StoreRequest(peer_sender, key, message) => (
            None,
            serve_store(ctx, peer_sender.addr, peer_sender.id, key, message).await,
        ),
        Command::FindValueRequest(peer_sender, key) => (
            None,
            serve_find_value(ctx, peer_sender.addr, peer_sender.id, key).await,
        ),
        Command::MessageRequest(message) => (None, serve_message(ctx, incoming_addr, message).await),
        Command::AnnounceRequest(peer_sender, crc) => (
            Some(peer_sender.id),
            serve_announce(ctx, peer_sender.addr, peer_sender.id, crc).await,
        ),
        Command::GetPeersRequest(crc) => (None, serve_get_peers(ctx, incoming_addr, crc).await),

        // Client message handling, shouldn't be reach.
        Command::ChunkResponse(_, _, _)
        | Command::FileInfoResponse(_)
        | Command::FindNodeResponse(_)
        | Command::PingResponse(_)
        | Command::ErrorOccured(_)
        | Command::StoreResponse()
        | Command::FindValueResponse(_)
        | Command::AnnounceResponse()
        | Command::GetPeersResponse(_)
        | Command::MessageResponse() => unreachable!(),
    };

    let response: Vec<u8> = res_command.into();

    let write_timeout = {
        let mut guard = main_ctx.lock().await;
        let ctx = guard.deref_mut();

        // Mark the peer who contact us, as alive.
        if let Some(sender) = sender {
            ctx.dht.peer_has_responded(sender).await;
        }

        // Check if we need to simulate a slowness.
        if let Some(wait_time) = ctx.slowness {
            sleep(wait_time).await;
        }

        ctx.write_timeout
    };

    // eprintln!("sending buf {:?}", &response);
    timeout(write_timeout, writer.write_all(response.as_slice())).await??;
    timeout(write_timeout, writer.flush()).await??;
    Ok(())
}

// Main handler ----------------------------------------------------------------

// Start to listen to command. One instance will be spawn for each peer.
pub async fn listen_to_command(
    ctx: Arc<Mutex<Context>>,
    mut stream: TcpStream,
    own_id: u32,
) -> AnyResult<()> {
    let read_timeout = {
        let guard = ctx.lock().await;
        let ctx = guard.deref();
        ctx.read_timeout
    };

    let peer_addr = stream.peer_addr()?;
    // eprintln!("{} is connected", peer_addr);
    let (reader, writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    loop {
        let raw_order = read_all!(reader, read_timeout);
        let len = raw_order.len();
        if len == 0 {
            break;
        }

        match raw_order.as_slice().try_into() {
            Ok(command) => dispatch(Arc::clone(&ctx), peer_addr, &mut writer, command, own_id).await?,
            Err(err) => eprintln!("Unknown command received! {}", err),
        }
    }

    Ok(())
}
