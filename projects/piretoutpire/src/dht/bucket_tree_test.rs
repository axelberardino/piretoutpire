use super::*;
use errors::AnyResult;

#[test]
fn test_construct_table() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;

    let mut tree = BucketTree::new();
    // Insert any value
    assert!(tree.add_peer_node(PeerNode::new(1000, dummy_addr)));
    // Ensure it can't be inserted twice
    assert!(!tree.add_peer_node(PeerNode::new(1000, dummy_addr)));

    // Insert as many values as to fill the bucket
    for idx in 1..BUCKET_SIZE {
        assert!(tree.add_peer_node(PeerNode::new(1000 + idx as u32, dummy_addr)));
    }

    for idx in 0..BUCKET_SIZE {
        assert!(tree.add_peer_node(PeerNode::new(1050 + idx as u32, dummy_addr)));
    }

    dbg!(&tree);

    Ok(())
}
