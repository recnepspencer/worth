//! Immutable, bounded input histories; values and costs come only from the seed.
use super::expected_history::{Member, Partition, World};

#[derive(Clone, Copy, Debug)]
pub(super) enum Family {
    Empty,
    Singleton,
    Chain,
    Diamond,
    Independent,
    Unequal,
    Nested,
}
pub(super) const FAMILIES: [Family; 7] = [
    Family::Empty,
    Family::Singleton,
    Family::Chain,
    Family::Diamond,
    Family::Independent,
    Family::Unequal,
    Family::Nested,
];
pub(super) const SEED: u64 = 0x9176_378a_0001;

pub(super) fn world(family: Family, seed: u64) -> World {
    let links: Vec<Vec<usize>> = match family {
        Family::Empty => vec![],
        Family::Singleton => vec![vec![]],
        Family::Chain => vec![vec![], vec![0], vec![1], vec![2]],
        Family::Diamond => vec![vec![], vec![0], vec![0], vec![1, 2]],
        Family::Independent | Family::Unequal | Family::Nested => vec![vec![]; 4],
    };
    let mut state = seed;
    let members: Vec<_> = links
        .into_iter()
        .enumerate()
        .map(|(index, upstream)| {
            let count = if matches!(family, Family::Nested) {
                4
            } else {
                1
            };
            let partitions = (0..count)
                .map(|part| {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    Partition {
                        key: part,
                        value: 1 + (state >> 32) % 64,
                        work: if matches!(family, Family::Unequal) {
                            1 + index as u64 * 7
                        } else {
                            2
                        },
                        fails: false,
                    }
                })
                .collect();
            Member {
                key: 8 - index as u64,
                upstream,
                partitions,
            }
        })
        .collect();
    let requested = (0..members.len()).rev().collect();
    World { members, requested }
}

#[test]
fn each_cell_obeys_the_operation_budget() {
    for family in FAMILIES {
        let world = world(family, SEED);
        assert!(world.requested.len() <= 8 && world.members.len() <= 8);
        for member in world.members {
            assert!(member.partitions.len() <= 4);
            assert!(member.partitions.iter().all(|part| part.work <= 128));
        }
    }
}
