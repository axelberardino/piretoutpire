use super::*;
use errors::AnyResult;

#[test]
fn test_construct_table() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;
    let middle = middle_point(0, u32::MAX) - BUCKET_SIZE as u32;

    let mut tree = BucketTree::new();
    // Insert any value
    assert!(tree.add_peer_node(PeerNode::new(middle, dummy_addr)));
    // Ensure it can't be inserted twice
    assert!(!tree.add_peer_node(PeerNode::new(middle, dummy_addr)));

    // Insert as many values as to fill the bucket
    for idx in 1..BUCKET_SIZE {
        assert!(tree.add_peer_node(PeerNode::new(middle + idx as u32, dummy_addr)));
    }
    assert!(tree.add_peer_node(PeerNode::new(middle + BUCKET_SIZE as u32, dummy_addr)));

    for idx in 0..BUCKET_SIZE + 1 {
        assert!(tree.add_peer_node(PeerNode::new((middle - 100) + idx as u32, dummy_addr)));
    }

    Ok(())
}
