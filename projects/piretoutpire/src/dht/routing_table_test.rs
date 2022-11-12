use super::*;
use errors::AnyResult;

#[tokio::test]
async fn test_init_table() -> AnyResult<()> {
    let dummy_addr = "127.0.0.1:4000".parse()?;

    let mut rt = RoutingTable::new(0);
    for id in 0..=10 {
        rt.add_node(PeerNode::new(id, dummy_addr)).await;
    }
    assert_eq!(11, rt.get_all_peers().await.count());

    assert_eq!(
        vec![0, 1, 2],
        rt.get_closest_peers_from(0, 3)
            .await
            .map(|peer| peer.id())
            .collect::<Vec<_>>()
    );

    assert_eq!(
        vec![1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 10],
        rt.get_closest_peers_from(1, 11)
            .await
            .map(|peer| peer.id())
            .collect::<Vec<_>>()
    );

    assert_eq!(
        vec![10, 8, 9, 2, 3, 0, 1, 6, 7, 4, 5],
        rt.get_closest_peers_from(10, 11)
            .await
            .map(|peer| peer.id())
            .collect::<Vec<_>>()
    );

    Ok(())
}
