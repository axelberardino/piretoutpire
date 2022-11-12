use super::*;
use errors::AnyResult;

#[tokio::test]
async fn test_construct_table() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;
    let middle = middle_point(0, u32::MAX) - BUCKET_SIZE as u32;

    let mut tree = BucketTree::new();
    // Insert any value
    assert!(tree.add_peer_node(PeerNode::new(middle, dummy_addr)).await);

    // Ensure it can't be inserted twice
    assert!(!tree.add_peer_node(PeerNode::new(middle, dummy_addr)).await);

    // Insert as many values as to fill the bucket
    for idx in 1..BUCKET_SIZE {
        assert!(
            tree.add_peer_node(PeerNode::new(middle + idx as u32, dummy_addr))
                .await
        );
    }

    assert!(
        tree.add_peer_node(PeerNode::new(middle + BUCKET_SIZE as u32, dummy_addr))
            .await
    );

    for idx in 0..BUCKET_SIZE {
        assert!(tree.add_peer_node(PeerNode::new(idx as u32, dummy_addr)).await);
    }

    Ok(())
}

#[tokio::test]
async fn test_closest_peers() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;
    let mut tree = BucketTree::new();

    // Insert as many values as to fill the bucket
    for idx in 0..=10 {
        assert!(tree.add_peer_node(PeerNode::new(idx as u32, dummy_addr)).await);
    }
    for idx in 15..=18 {
        assert!(tree.add_peer_node(PeerNode::new(idx as u32, dummy_addr)).await);
    }

    let peers = tree.search_closest_peers(100).await;
    assert_eq!(15, peers.count());

    Ok(())
}
