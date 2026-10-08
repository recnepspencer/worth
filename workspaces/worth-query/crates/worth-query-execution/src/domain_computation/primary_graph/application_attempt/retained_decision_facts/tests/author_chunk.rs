//! Active-chunk full-key insertion preserves canonical and arrival ordering.
use super::super::{restore_authored_order, RetainedFactStore, StoreDenial};
use super::{control, key, merge, Body};
use worth_execution::ExecutionAllocationPolicy as Policy;

#[test]
fn active_chunk_orders_full_keys_and_resolves_boundary_and_interior_duplicates() {
    let policy = control(Policy::SystemAllocation);
    let authored = [6, 2, 10, 1, 7, 3, 5, 0, 12];
    let populate = || {
        let mut store = RetainedFactStore::new(policy).unwrap();
        for limit in authored {
            store
                .insert(
                    key(b"same", Some((b"predicate", limit)), policy),
                    Body {
                        scalar: limit as u64,
                        roles: 1,
                    },
                    policy,
                    merge,
                )
                .unwrap();
        }
        store
    };
    let mut store = populate();
    for limit in [0, 5, 12] {
        store
            .insert(
                key(b"same", Some((b"predicate", limit)), policy),
                Body {
                    scalar: limit as u64,
                    roles: 2,
                },
                policy,
                merge,
            )
            .unwrap();
    }
    let canonical = store.finish(policy).unwrap();
    assert_eq!(canonical.len(), 9);
    for (record, limit) in canonical.iter().zip([0, 1, 2, 3, 5, 6, 7, 10, 12]) {
        assert_eq!(record.key.locator(), b"same");
        assert_eq!(
            record.key.predicate_material(),
            Some((b"predicate".as_slice(), limit))
        );
        assert_eq!(record.value.scalar, limit as u64);
        assert_eq!(
            record.value.roles,
            if [0, 5, 12].contains(&limit) { 3 } else { 1 }
        );
    }
    let restored = restore_authored_order(canonical, policy).unwrap();
    for (ordinal, (record, limit)) in restored.iter().zip(authored).enumerate() {
        assert_eq!(record.ordinal, ordinal);
        assert_eq!(record.value.scalar, limit as u64);
    }
    for limit in [0, 5, 12] {
        let mut conflict = populate();
        assert!(matches!(
            conflict.insert(
                key(b"same", Some((b"predicate", limit)), policy),
                Body {
                    scalar: 99,
                    roles: 2
                },
                policy,
                merge,
            ),
            Err(StoreDenial::ConflictingBody)
        ));
        assert!(matches!(
            conflict.insert(
                key(b"different", None, policy),
                Body {
                    scalar: 13,
                    roles: 1
                },
                policy,
                merge,
            ),
            Err(StoreDenial::ConflictingBody)
        ));
        assert!(matches!(
            conflict.finish(policy),
            Err(StoreDenial::ConflictingBody)
        ));
    }
}
