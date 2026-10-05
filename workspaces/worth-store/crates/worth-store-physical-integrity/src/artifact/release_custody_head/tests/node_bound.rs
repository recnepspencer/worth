//! The caller's node bound is its own denial: reaching it says how many
//! blocks passed the bound, and nothing about the tree's integrity.

use super::*;

/// Three single-entry leaves under one branch: four blocks hold three heads,
/// so a bound counted in heads is passed by a tree nothing is wrong with.
struct SparseTree {
    leaves: [ReleaseCustodyHeadBlockV1; 3],
    branch: ReleaseCustodyHeadBlockV1,
    root: DurablePhysicalRootManifest,
}

impl SparseTree {
    fn new() -> Self {
        let leaves = [1_u8, 2, 3].map(|object| {
            ReleaseCustodyHeadBlockV1::leaf(9, 2, u64::from(object), vec![entry(object)], format())
                .unwrap()
        });
        let children = leaves.iter().map(|leaf| leaf.reference(format())).collect();
        let branch = ReleaseCustodyHeadBlockV1::branch(9, 2, 4, 1, children, format()).unwrap();
        let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
            .release_custody_head_root(Some(branch.reference(format())))
            .next_release_custody_head_block(5)
            .admit()
            .unwrap();
        Self {
            leaves,
            branch,
            root,
        }
    }

    fn frame(&self, block: u64) -> Vec<u8> {
        match block {
            4 => self.branch.encode(format()),
            leaf => self.leaves[leaf as usize - 1].encode(format()),
        }
    }

    /// Walks under `max_nodes`, with `damaged` served one byte off.
    fn walk(
        &self,
        max_nodes: u64,
        damaged: Option<u64>,
    ) -> (
        Result<ReleaseCustodyHeadWalkV1, ReleaseCustodyHeadWalkDenial<(), ()>>,
        Vec<u64>,
    ) {
        let limits =
            ReleaseCustodyHeadWalkLimitsV1::new(max_nodes, 3, 8 * 16_384, 128 * 1024, 2).unwrap();
        let mut reads = Vec::new();
        let walked = walk_release_custody_head(
            &self.root,
            format(),
            limits,
            |reference, _| {
                reads.push(reference.block());
                let mut frame = self.frame(reference.block());
                if damaged == Some(reference.block()) {
                    let last = frame.len() - 1;
                    frame[last] ^= 1;
                }
                Ok::<_, ()>(frame)
            },
            |_| Ok::<_, ()>(()),
        );
        (walked, reads)
    }
}

#[test]
fn a_sound_tree_of_more_blocks_than_the_bound_is_that_bound_with_its_count() {
    let tree = SparseTree::new();
    // The bound is the tree's three heads; its four blocks pass it.
    let (walked, reads) = tree.walk(3, None);
    assert_eq!(
        walked,
        Err(ReleaseCustodyHeadWalkDenial::NodeBound {
            observed: 4,
            admitted: 3,
        })
    );
    // The branch said how many blocks it has; no leaf was read past the bound.
    assert_eq!(reads, vec![4]);
}

#[test]
fn the_same_tree_walks_whole_under_a_bound_of_its_block_count() {
    let tree = SparseTree::new();
    let (walked, reads) = tree.walk(4, None);
    let walked = walked.unwrap();
    assert_eq!((walked.node_count(), walked.entry_count()), (4, 3));
    assert_eq!(reads, vec![4, 1, 2, 3]);
}

#[test]
fn a_damaged_block_under_a_generous_bound_is_damage_not_the_bound() {
    let tree = SparseTree::new();
    let (walked, reads) = tree.walk(64, Some(2));
    assert!(
        matches!(walked, Err(ReleaseCustodyHeadWalkDenial::Format(_))),
        "{walked:?}",
    );
    assert_eq!(reads, vec![4, 1, 2]);
}
