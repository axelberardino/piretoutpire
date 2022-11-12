use super::peer_node::PeerNode;
use crate::{dht::peer_node::PeerStatus, utils::middle_point};
use std::{cell::RefCell, rc::Rc};

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
    root: Rc<RefCell<TreeNode>>,
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
    Children(Rc<RefCell<TreeNode>>, Bucket),
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
            root: Rc::new(RefCell::new(TreeNode {
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
    pub fn add_peer_node(&mut self, peer_node: PeerNode) -> bool {
        let rc_tree_node = self.find_leaf(peer_node.id());
        let mut tree_node = rc_tree_node.borrow_mut();
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
        let mut rc_tree_node = Rc::clone(&rc_tree_node);
        loop {
            let (start, end) = {
                let tree = rc_tree_node.borrow();
                (tree.start, tree.end)
            };

            let (split, new_left, new_right) = split_node(Rc::clone(&rc_tree_node), start, end);
            let new_node = if peer_id < split { new_left } else { new_right };

            let succeed = match &mut new_node.borrow_mut().children {
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
    pub fn search_closest_peers(&self, nb: usize) -> Vec<PeerNode> {
        let mut queue = Vec::new();
        let mut res = Vec::new();
        queue.push(Rc::clone(&self.root));
        while let Some(rc_tree_node) = queue.pop() {
            let tree_node = rc_tree_node.borrow();
            match &tree_node.children {
                LeafOrChildren::Leaf(bucket) => {
                    res.extend(bucket.peers.iter().map(Clone::clone).collect::<Vec<_>>());
                    if res.len() >= nb {
                        return res;
                    }
                }
                LeafOrChildren::Children(rc_left, bucket) => {
                    res.extend(bucket.peers.iter().map(Clone::clone).collect::<Vec<_>>());
                    if res.len() >= nb {
                        return res;
                    }
                    queue.push(Rc::clone(rc_left));
                }
            }
        }

        res
    }
}

// Private methods.
impl BucketTree {
    // Search the corresponding leaf.
    fn find_leaf(&self, id: u32) -> Rc<RefCell<TreeNode>> {
        fn rec_find_leaf(rc_bucket_tree: Rc<RefCell<TreeNode>>, id: u32) -> Rc<RefCell<TreeNode>> {
            let bucket_tree = rc_bucket_tree.borrow();
            debug_assert!(id >= bucket_tree.start);
            debug_assert!(id < bucket_tree.end);

            match &bucket_tree.children {
                LeafOrChildren::Leaf(_) => Rc::clone(&rc_bucket_tree),
                LeafOrChildren::Children(rc_left, _) => {
                    let left = rc_left.borrow();
                    if id < left.end {
                        rec_find_leaf(Rc::clone(rc_left), id)
                    } else {
                        Rc::clone(&rc_bucket_tree)
                    }
                }
            }
        }
        rec_find_leaf(Rc::clone(&self.root), id)
    }
}

// Split an existing node in two. Cut the given range in half and move peers in
// left or right bucket.
// Return the value on which to split, and the left and right node created.
fn split_node(
    rc_bucket_node: Rc<RefCell<TreeNode>>,
    start: u32,
    end: u32,
) -> (u32, Rc<RefCell<TreeNode>>, Rc<RefCell<TreeNode>>) {
    let mut bucket_node = rc_bucket_node.borrow_mut();
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

    let left = Rc::new(RefCell::new(TreeNode {
        start,
        end: split_on,
        children: LeafOrChildren::Leaf(Bucket { peers: left_peers }),
    }));
    let right = Bucket { peers: right_peers };

    bucket_node.children = LeafOrChildren::Children(Rc::clone(&left), right);

    (split_on, left, Rc::clone(&rc_bucket_node))
}

#[cfg(test)]
#[path = "bucket_tree_test.rs"]
mod bucket_tree_test;
