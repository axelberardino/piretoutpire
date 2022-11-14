use super::*;
use std::collections::HashMap;

// MOCKED FUNCTIONS ------------------------------------------------------------

// Emulate the querying of another peer with find_node.
async fn mock_find_node(
    peers: HashMap<u32, Vec<u32>>,
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let mut nodes = peers.get(&peer.id).map_or(vec![], |vec| {
        vec.into_iter()
            .map(|peer_id| Peer {
                id: *peer_id,
                addr: "127.0.0.1:4000".parse().expect(""),
            })
            .collect()
    });
    nodes.sort_by_key(|peer| distance(peer.id, target));

    // The peer just answered us, let's add him into our dht.
    {
        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        ctx.dht.add_node(peer.id, peer.addr).await;
    }

    Ok(nodes.into_iter().take(4).collect())
}

// This function own a predefined set of nodes, emulating a dht network.
// 1 -- 2
// | \ /
// 5  3 - 4 - 7
//     \      |
// 9    6     8
async fn mocked_query_find_node_small(
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    _sender: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let mut peers = HashMap::<u32, Vec<u32>>::new();
    peers.insert(1, vec![2, 3, 5]);
    peers.insert(2, vec![1, 3]);
    peers.insert(3, vec![1, 2, 4, 6]);
    peers.insert(4, vec![3, 7]);
    peers.insert(5, vec![1]);
    peers.insert(6, vec![3]);
    peers.insert(7, vec![4, 8]);
    peers.insert(8, vec![7]);
    peers.insert(9, vec![]);
    mock_find_node(peers, ctx, peer, target).await
}

// A big mock-up with many interconnected nodes.
async fn mocked_query_find_node_big(
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    _sender: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let mut peers = HashMap::<u32, Vec<u32>>::new();
    peers.insert(1, vec![34, 43, 49, 60, 16, 18, 19, 12, 13, 15, 4, 5, 6]);
    peers.insert(4, vec![34, 43, 49, 60, 16, 18, 19, 12, 13, 15, 1, 6, 5]);
    peers.insert(5, vec![34, 43, 49, 60, 16, 18, 19, 12, 13, 15, 1, 6]);
    peers.insert(6, vec![34, 43, 49, 60, 16, 18, 19, 12, 13, 15, 1, 4, 5]);
    peers.insert(12, vec![34, 43, 49, 60, 16, 18, 19, 1, 4, 5, 6, 15, 13]);
    peers.insert(13, vec![34, 43, 49, 60, 16, 18, 19, 1, 4, 5, 6, 15, 12]);
    peers.insert(15, vec![34, 43, 49, 60, 16, 18, 19, 1, 4, 5, 6, 12, 13]);
    peers.insert(16, vec![34, 43, 49, 60, 1, 4, 5, 6, 18, 19]);
    peers.insert(18, vec![34, 43, 49, 60, 1, 4, 5, 6, 16, 19]);
    peers.insert(19, vec![34, 43, 49, 60, 1, 4, 5, 6, 16, 18]);
    peers.insert(34, vec![1, 4, 5, 6, 49, 60, 62, 43]);
    peers.insert(43, vec![1, 4, 5, 6, 49, 60, 62, 34]);
    peers.insert(49, vec![1, 4, 5, 6, 34, 43, 60, 62]);
    peers.insert(60, vec![1, 4, 5, 6, 34, 43, 49, 62]);
    peers.insert(62, vec![1, 4, 5, 6, 34, 43, 49, 60]);
    mock_find_node(peers, ctx, peer, target).await
}

// A big mock-up with some interconnected nodes.
async fn mocked_query_find_node_partial(
    ctx: Arc<Mutex<Context>>,
    peer: Peer,
    _sender: u32,
    target: u32,
) -> AnyResult<Vec<Peer>> {
    let mut peers = HashMap::<u32, Vec<u32>>::new();
    peers.insert(1, vec![4, 5, 6]);
    peers.insert(4, vec![13, 15, 1, 6, 5]);
    peers.insert(5, vec![12, 13, 15, 1, 6]);
    peers.insert(6, vec![12, 13, 15, 1, 4, 5]);
    peers.insert(12, vec![18, 19, 4, 5]);
    peers.insert(13, vec![16, 18, 19, 1, 4, 5, 6, 15, 12]);
    peers.insert(15, vec![16, 18, 19, 1, 4, 5, 6, 12, 13]);
    peers.insert(16, vec![34, 43, 49, 60, 1, 4, 5, 6, 18, 19]);
    peers.insert(18, vec![34, 43, 49, 60, 1, 4, 5, 6, 16, 19]);
    peers.insert(19, vec![34, 43, 49, 60, 1, 4, 5, 6, 16, 18]);
    peers.insert(34, vec![1, 4, 5, 6, 49, 60, 62, 43]);
    peers.insert(43, vec![1, 4, 5, 6, 49, 60, 62, 34]);
    peers.insert(49, vec![1, 4, 5, 6, 34, 43, 60, 62]);
    peers.insert(60, vec![1, 4, 5, 6, 34, 43, 49, 62]);
    peers.insert(62, vec![1, 4, 5, 6, 34, 43, 49, 60]);
    mock_find_node(peers, ctx, peer, target).await
}

// TESTS -----------------------------------------------------------------------

#[tokio::test]
async fn test_find_node_itself() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 0;
    let target = 0;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            ctx,
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_small,
        )
        .await?;
        assert!(res.is_none());
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            ctx,
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_small,
        )
        .await?;
        assert!(res.is_none());
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_1_roundtrip() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 1;
    let target = 2;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_small,
        )
        .await?;
        assert_eq!(target, res.expect("should be there").id);

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        assert_eq!(vec![1, 2, 3, 5], ctx.dht.peer_ids().await);
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_small,
        )
        .await?;
        assert_eq!(target, res.expect("should be there").id);

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        assert_eq!(vec![1, 2], ctx.dht.peer_ids().await);
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_max_roundtrip() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 8;
    let target = 1;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_small,
        )
        .await?;
        assert_eq!(target, res.expect("should be there").id);
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_small,
        )
        .await?;
        assert_eq!(target, res.expect("should be there").id);
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_unbalanced_roundtrip() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 1;
    let target = 8;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_small,
        )
        .await?;
        // 8 is there, and there is a path to it.
        // Although, very few nodes knows about it, and finding nodes is quickly
        // interrupted because the distance is too far away.
        // Distance would look like this:
        // 9 --- 10
        // | \  /
        // 13 11 - [12] - 15
        //     \        |
        // 1   14       0
        //
        // Meaning it will block at node 12 because it's not closer than the best (10).
        assert!(res.is_none());
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_small,
        )
        .await?;
        // Using the hop strategy, we will succed to find the node, we would
        // have missed with the classic algorithm in a graph with few
        // participants.
        assert_eq!(target, res.expect("should be there").id);
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_max_roundtrip_in_a_big_mock_not_found() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 1;
    let target = 47;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_big,
        )
        .await?;
        // Will be not found, closest should be 43.
        assert!(res.is_none());

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        // Node 1 (starting point) and its next 3 nodes (34, 43, 60) should be added
        // in the dht.
        assert_eq!(vec![1, 34, 43, 60], ctx.dht.peer_ids().await);
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_big,
        )
        .await?;
        // Will be not found, closest should be 43.
        assert!(res.is_none());

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        // Should be [1, 34, 43, 49, 60, 62], but the routing table is full so
        // 49 is not added.
        assert_eq!(vec![1, 34, 43, 60, 62], ctx.dht.peer_ids().await);
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_max_roundtrip_in_a_big_mock_found() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 1;
    let target = 43;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_big,
        )
        .await?;
        // 43 will be found!
        assert_eq!(target, res.expect("should be there").id);

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        // Node 1 (starting point) and its next 3 nodes (34, 43, 60) should be added
        // in the dht.
        assert_eq!(vec![1, 34, 43, 60], ctx.dht.peer_ids().await);
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_big,
        )
        .await?;
        // 43 will be found!
        assert_eq!(target, res.expect("should be there").id);

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        // Node 1 (starting point) and only the found node.
        assert_eq!(vec![1, 43], ctx.dht.peer_ids().await);
    }

    Ok(())
}

#[tokio::test]
async fn test_find_node_max_roundtrip_in_a_partial_mock_found() -> AnyResult<()> {
    let sender = 0;
    let starting_from = 1;
    let target = 43;

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            None,
            mocked_query_find_node_partial,
        )
        .await?;
        // 43 will not be found, because the graph is too much partial.
        assert!(res.is_none());

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        // Node 1 (starting point) and its next 3 nodes (34, 43, 60) should be added
        // in the dht.
        assert_eq!(vec![1, 4, 5, 6, 12, 13, 15], ctx.dht.peer_ids().await);
    }

    {
        let ctx = Arc::new(Mutex::new(Context::new("".to_owned(), sender)));
        let res = find_closest_node(
            Arc::clone(&ctx),
            Peer {
                id: starting_from,
                addr: "127.0.0.1:4000".parse()?,
            },
            sender,
            target,
            Some(u32::MAX),
            mocked_query_find_node_partial,
        )
        .await?;
        // 43 will be found!
        assert_eq!(target, res.expect("should be there").id);

        let mut guard = ctx.lock().await;
        let ctx = guard.deref_mut();
        assert_eq!(vec![1, 4, 5, 6, 12, 13, 15, 18, 19, 43], ctx.dht.peer_ids().await);
    }

    Ok(())
}
