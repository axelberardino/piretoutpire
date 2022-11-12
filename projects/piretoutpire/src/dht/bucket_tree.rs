use super::peer_node::PeerNode;
use crate::{dht::peer_node::PeerStatus, utils::middle_point};
use std::sync::Arc;
use tokio::sync::Mutex;

// Maximum nodes by bucket. Bittorent use 8.
const BUCKET_SIZE: usize = 4;

// Allow to store data in an unbalanced tree with dynamic bucketing. There are
// more nodes on the left, than on the right.
//
// Everytime we insert a value, if we have more than BUCKET_SIZE value, the
// current bucket will be split into 2. The bucket id value is div by 2, and all
// values are distributed in the left or right sub-bucket accordingly.
//
// This is what a tree would looks like, with a range of [0, 31], with value:
// [0, 1, 4, 5, 6, 8, 9, 16, 25, 30, 31] and with a bucket size of 4.
//
//            (16)
//            / \
//         (8)   [16, 25, 30, 31]
//         / \
//      (4)   [8, 9]
//      / \
// [0, 1] [4, 5, 6]
//
// Note that trying to add a new node in a full bucket, either result in the
// tree to split this node and add it, or discard the new node because there is
// no more room.
// Because we allow more node on the left, it means this tree will store more
// values close to 0.
#[derive(Debug)]
pub struct BucketTree {
    // Rc<RefCell<TreeNode>> would have been enough, but this dataset is used in
    // an async environment. So arc/mutex it is :(.
    root: Arc<Mutex<TreeNode>>,
}

#[derive(Debug)]
pub struct TreeNode {
    // Start of the range
    start: u32,
    // End of the range
    end: u32,
    // Successor
    children: LeafOrChildren,
}

#[derive(Debug)]
enum LeafOrChildren {
    Leaf(Bucket),
    Children(Arc<Mutex<TreeNode>>, Bucket),
}

#[derive(Debug)]
struct Bucket {
    // List of all peers in the bucket. Their id must be between start and end.
    peers: Vec<PeerNode>,
    // ??
    // freshness: ?
}

// Public interface.
impl BucketTree {
    // Initialize a new tree.
    pub fn new() -> Self {
        Self {
            root: Arc::new(Mutex::new(TreeNode {
                start: 0,
                end: u32::MAX,
                children: LeafOrChildren::Leaf(Bucket {
                    peers: Vec::with_capacity(BUCKET_SIZE),
                }),
            })),
        }
    }

    // Add a new peer info into the tree.
    // Returns if an insertion has been made.
    pub async fn add_peer_node(&mut self, peer_node: PeerNode) -> bool {
        let rc_tree_node = self.find_leaf(peer_node.id()).await;
        let mut tree_node = rc_tree_node.lock().await;
        debug_assert!(peer_node.id() >= tree_node.start);
        debug_assert!(peer_node.id() < tree_node.end);
        let (bucket, right_leaf) = match &mut tree_node.children {
            LeafOrChildren::Leaf(bucket) => (bucket, false),
            LeafOrChildren::Children(_, bucket) => (bucket, true),
        };

        // Already exists
        if bucket.peers.iter().any(|peer| peer.id() == peer_node.id()) {
            return false;
        }

        // Enough room for a new peer
        if bucket.peers.len() < BUCKET_SIZE {
            bucket.peers.push(peer_node);
            bucket.peers.sort_by_key(|peer| peer.id());
            return true;
        }

        // Not enough room, try to replace a bad peer first
        if let Some(bad_node) = bucket
            .peers
            .iter_mut()
            .find(|peer| peer.status() == PeerStatus::Bad)
        {
            *bad_node = peer_node;
            bucket.peers.sort_by_key(|peer| peer.id());
            return true;
        }

        // We're already on a right leaf, and there's no room, just give up.
        if right_leaf {
            return false;
        }

        // Start by releasing all borrowed values.
        let peer_id = peer_node.id();
        drop(tree_node);

        // Bucket is full, no other choice but to split it, and add the new
        // value in one of its new children.
        // The loop is there to handle the case where splitting a range give:
        // One new node full + one new node empty. So we need to loop until it's
        // resolved.
        let mut rc_tree_node = Arc::clone(&rc_tree_node);
        loop {
            let (start, end) = {
                let tree = rc_tree_node.lock().await;
                (tree.start, tree.end)
            };

            let (split, new_left, new_right) = split_node(Arc::clone(&rc_tree_node), start, end).await;
            let new_node = if peer_id < split { new_left } else { new_right };

            let succeed = match &mut new_node.lock().await.children {
                LeafOrChildren::Leaf(bucket) => {
                    if bucket.peers.len() < BUCKET_SIZE {
                        bucket.peers.push(peer_node.clone());
                        bucket.peers.sort_by_key(|peer| peer.id());
                        true
                    } else {
                        false
                    }
                }
                LeafOrChildren::Children(_, bucket) => {
                    if bucket.peers.len() < BUCKET_SIZE {
                        bucket.peers.push(peer_node.clone());
                        bucket.peers.sort_by_key(|peer| peer.id());
                        true
                    } else {
                        return false;
                    }
                }
            };

            if succeed {
                return true;
            }
            if end - start <= BUCKET_SIZE as u32 {
                return false;
            }

            rc_tree_node = new_node;
        }
    }

    // Search the closest peers
    pub async fn search_closest_peers(&self, nb: usize) -> impl Iterator<Item = PeerNode> {
        let mut queue = Vec::new();
        let mut res = Vec::new();
        queue.push(Arc::clone(&self.root));
        while let Some(rc_tree_node) = queue.pop() {
            let tree_node = rc_tree_node.lock().await;
            match &tree_node.children {
                LeafOrChildren::Leaf(bucket) => {
                    res.extend(bucket.peers.iter().map(Clone::clone).collect::<Vec<_>>());
                    if res.len() >= nb {
                        return res.into_iter();
                    }
                }
                LeafOrChildren::Children(rc_left, bucket) => {
                    res.extend(bucket.peers.iter().map(Clone::clone).collect::<Vec<_>>());
                    if res.len() >= nb {
                        return res.into_iter();
                    }
                    queue.push(Arc::clone(rc_left));
                }
            }
        }

        res.into_iter()
    }

    // Return all contained peers.
    pub async fn get_all_peers(&self) -> impl Iterator<Item = PeerNode> {
        self.search_closest_peers(usize::MAX).await
    }
}

// Private methods.
impl BucketTree {
    // Search the corresponding leaf.
    async fn find_leaf(&self, id: u32) -> Arc<Mutex<TreeNode>> {
        let mut queue = Vec::new();
        queue.push(Arc::clone(&self.root));
        while let Some(rc_tree_node) = queue.pop() {
            let tree_node = rc_tree_node.lock().await;
            match &tree_node.children {
                LeafOrChildren::Leaf(_) => {
                    return Arc::clone(&rc_tree_node);
                }
                LeafOrChildren::Children(rc_left, _) => {
                    let left = rc_left.lock().await;
                    if id < left.end {
                        queue.push(Arc::clone(rc_left));
                    } else {
                        return Arc::clone(&rc_tree_node);
                    }
                }
            }
        }

        unreachable!()
    }
}

// Split an existing node in two. Cut the given range in half and move peers in
// left or right bucket.
// Return the value on which to split, and the left and right node created.
async fn split_node(
    rc_bucket_node: Arc<Mutex<TreeNode>>,
    start: u32,
    end: u32,
) -> (u32, Arc<Mutex<TreeNode>>, Arc<Mutex<TreeNode>>) {
    let mut bucket_node = rc_bucket_node.lock().await;
    let bucket = match &mut bucket_node.children {
        LeafOrChildren::Leaf(bucket) => bucket,
        LeafOrChildren::Children(_, _) => unreachable!(),
    };

    let split_on = middle_point(start, end);
    let (left_peers, right_peers) = bucket
        .peers
        .drain(..)
        .fold((Vec::new(), Vec::new()), |mut acc, peer| {
            if peer.id() < split_on {
                acc.0.push(peer);
            } else {
                acc.1.push(peer)
            }
            acc
        });

    let left = Arc::new(Mutex::new(TreeNode {
        start,
        end: split_on,
        children: LeafOrChildren::Leaf(Bucket { peers: left_peers }),
    }));
    let right = Bucket { peers: right_peers };

    bucket_node.children = LeafOrChildren::Children(Arc::clone(&left), right);

    (split_on, left, Arc::clone(&rc_bucket_node))
}

#[cfg(test)]
#[path = "bucket_tree_test.rs"]
mod bucket_tree_test;
