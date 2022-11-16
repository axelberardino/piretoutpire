use super::{client::handle_find_node, context::Context};
use crate::{network::protocol::Peer, utils::distance};
use errors::{AnyError, AnyResult};
use std::{collections::HashSet, future::Future, ops::DerefMut, sync::Arc};
use tokio::{
    self,
    net::TcpStream,
    sync::Mutex,
    time::{sleep, timeout},
};

// Search for a requested node until finding it. Will stop if the most closest
// ones found in a row are not closer.
// Return either the found peer or none.
pub async fn find_closest_node<F, T>(
    ctx: Arc<Mutex<Context>>,
    initial_peer: Peer,
    sender: u32,
    target: u32,
    max_hop: Option<u32>,
    query_func: F,
) -> AnyResult<Option<Peer>>
where
    F: FnMut(Arc<Mutex<Context>>, Peer, u32, u32) -> T + Send + Copy + 'static,
    T: Future<Output = AnyResult<Vec<Peer>>> + Send + 'static,
{
    let mut hop = 0;
    let mut queue = vec![initial_peer];
    let mut visited = HashSet::<u32>::new();
    visited.insert(sender); // Let's avoid ourself.
    let mut best_distance = u32::MAX;
    let mut found_peer = None::<Peer>;

    loop {
        hop += 1;

        // Just launch 3 find_node at the same time, with the first 3
        // non-visited peers in the queue. Will drain peer from the queue, until
        // the queue is empty or 3 non visited has been queried.
        // Will responsd with a queue containing from 0 up to 3*4 uniques nodes.
        let next_queue = parallel_find_node(
            Arc::clone(&ctx),
            sender,
            target,
            &mut queue,
            &mut visited,
            query_func,
            3,
        )
        .await?;

        // Let's check if the best peers is better than the previous hop.
        let mut better_distance_found = false;
        if let Some(peer) = next_queue.first() {
            let distance = distance(peer.id, target);
            if distance < best_distance {
                best_distance = distance;
                better_distance_found = true;
            }
            if distance == 0 {
                found_peer = Some(peer.clone());
            }
        }

        queue.extend(next_queue);
        // Now let's sort the queue putting the best at the end (easier for
        // poping values).
        queue.sort_by_key(|peer| distance(peer.id, target));
        queue.reverse();
        queue.dedup_by_key(|peer| peer.id);

        // Handle strategy here.
        if let Some(max_hop) = max_hop {
            // If we found the exact peer, put it in our dht, and stop searching.
            if let Some(peer) = &found_peer {
                let mut guard = ctx.lock().await;
                let ctx = guard.deref_mut();
                ctx.dht.add_node(peer.id, peer.addr).await;
                break;
            }
            // Hop strategy: stop after doing N hops, or if we found the target.
            // If all nodes have been visited, also stop.
            if hop >= max_hop || queue.is_empty() {
                break;
            }
        } else {
            // Classic strategy: stop when the next route is not closer to the
            // target than this one.
            // If the next group queried didn't return a better result, we stop
            // to hop.
            if !better_distance_found {
                break;
            }
        }
    }

    Ok(found_peer)
}

// Will drain N values from the main task queues, launch them in parallel, then
// return a flatten results of peers. Peers will be sorted by relevancy (closest
// first).
async fn parallel_find_node<F, T>(
    ctx: Arc<Mutex<Context>>,
    sender: u32,
    target: u32,
    queue: &mut Vec<Peer>,
    visited: &mut HashSet<u32>,
    mut query_func: F,
    nb_parallel: usize,
) -> AnyResult<Vec<Peer>>
where
    F: FnMut(Arc<Mutex<Context>>, Peer, u32, u32) -> T + Send + Copy + 'static,
    T: Future<Output = AnyResult<Vec<Peer>>> + Send + 'static,
{
    let mut next_queue = Vec::new();

    // Just launch 3 concurrent tasks.
    let mut queries = Vec::new();
    let mut nb_tasks = 0;

    while let Some(peer) = queue.pop() {
        if visited.contains(&peer.id) {
            continue;
        }

        nb_tasks += 1;
        let ctx = Arc::clone(&ctx);
        let handle = tokio::spawn(async move {
            let peer_id = peer.id;
            let peers = query_func(ctx, peer, sender, target).await?;
            Ok::<(u32, Vec<Peer>), AnyError>((peer_id, peers))
        });

        queries.push(handle);
        if nb_tasks >= nb_parallel {
            break;
        }
    }

    // Now wait for all tasks to complete and put result in a queue.
    for handle in queries {
        let (peer_id, peers) = handle.await??;
        visited.insert(peer_id);
        // Let's keep the 4 best nodes found.
        next_queue.extend(peers.into_iter());
    }

    // Keep only non-visited nodes.
    next_queue.retain(|peer| !visited.contains(&peer.id));

    // Sort peers by relevancy (the closest first)
    next_queue.sort_by_key(|peer| distance(peer.id, target));
    next_queue.dedup_by_key(|peer| peer.id);

    Ok(next_queue)
}

// Query the distant nodes and update the current context.
pub async fn query_find_node(
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    sender: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let (slowness, connection_timeout) = {
        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        (ctx.slowness, ctx.connection_timeout)
    };

    let connexion = timeout(connection_timeout, TcpStream::connect(peer.addr)).await;
    match connexion {
        Ok(Ok(connexion)) => {
            let stream = Arc::new(Mutex::new(connexion));
            if let Some(wait_time) = slowness {
                sleep(wait_time).await;
            }
            let peers = handle_find_node(Arc::clone(&ctx), stream, sender, target).await?;

            // The peer just answered us, let's add him into our dht.
            {
                let mut guard = ctx.lock().await;
                let ctx = guard.deref_mut();
                ctx.dht.add_node(peer.id, peer.addr).await;
            }

            Ok(peers)
        }
        _ => {
            // Peer is not connected, or timeout. Just ignore it.
            Ok(vec![])
        }
    }
}

#[cfg(test)]
#[path = "find_node_test.rs"]
mod find_node_test;
