//! Actual equality-index calls depend on merged unique writes, not target size.
use super::*;
use crate::domain_computation::primary_graph::merge_unique_values::take_lookup_limits;
#[test]
fn merge_unique_lookup_count_is_bounded_by_its_writes() {
    for unrelated in [0, 256] {
        let world = MergeWorld::new();
        for item in 0..unrelated {
            world.create("main", &format!("unrelated-{item}"), item);
        }
        world.fork();
        let writes = 3;
        for item in 0..writes {
            world.create(FEATURE, &format!("merged-{item}"), 1000 + item);
        }
        take_lookup_limits();
        assert_eq!(world.merge(), Ok(()));
        assert_eq!(take_lookup_limits(), vec![2; writes as usize]);
    }
}
