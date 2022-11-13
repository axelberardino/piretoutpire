use super::*;

// Query the distant nodes and update the current context.
async fn mocked_query_find_node(
    _ctx: Arc<Mutex<Context>>,
    _peer: Peer,
    _sender: u32,
    _target: u32,
) -> AnyResult<Vec<Peer>> {
    let peers = vec![];

    Ok(peers)
}

#[test]
fn test_find_closest_node() -> AnyResult<()> {
    Ok(())
}
