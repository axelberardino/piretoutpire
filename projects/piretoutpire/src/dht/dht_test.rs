use super::*;
use errors::AnyResult;

#[tokio::test]
async fn test_add_node() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;
    let dummy_peer = PeerNode::new(0, dummy_addr);

    let mut dht = DistributedHashTable::new(0);
    assert_eq!(0, dht.len().await);
    dht.add_node(dummy_peer.clone()).await;
    assert_eq!(1, dht.len().await);

    Ok(())
}

#[tokio::test]
async fn test_find_node() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;
    let dummy_peer = PeerNode::new(0, dummy_addr);

    let mut dht = DistributedHashTable::new(0);

    // dht is initialy empty.
    assert_eq!(0, dht.len().await);
    // Try to find a non-existing entry. Entry will be not found...
    assert_eq!(
        Vec::<PeerNode>::new(),
        dht.find_node(dummy_peer.clone(), 38).await.collect::<Vec<_>>()
    );
    // ... but the sender will be added into the dht.
    assert_eq!(1, dht.len().await);

    dht.add_node(PeerNode::new(0, dummy_addr)).await;

    Ok(())
}
