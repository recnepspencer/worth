//! Draft storage proof only. Not an installed Query admission or scale proof.
//! Run in a dedicated test binary so its authority does not compete with other
//! test targets' process authority; linking/runtime remain root-owned.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::handoff::restore_authored_order;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::canonical_key::AdmittedFactKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::storage::RetainedFactStore;
use worth_query_execution::facade::application_inspection::StoreDenial;
use worth_execution::ExecutionAllocationPolicy as Policy;

pub struct Body {
    pub scalar: u64,
    pub roles: u8,
}

pub fn key(
    locator: &[u8],
    predicate: Option<(&[u8], usize)>,
    policy: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
) -> AdmittedFactKey {
    AdmittedFactKey::write_for_test(
        locator.len(),
        predicate.map(|(bytes, limit)| (bytes.len(), limit)),
        policy,
        |writer| Ok(writer.extend_from_slice(locator)?),
        |writer| Ok(writer.extend_from_slice(predicate.unwrap().0)?),
    )
    .unwrap()
}
pub fn merge(existing: &mut Body, incoming: Body) -> Result<(), StoreDenial> {
    if existing.scalar != incoming.scalar {
        return Err(StoreDenial::ConflictingBody);
    }
    existing.roles |= incoming.roles;
    Ok(())
}

#[test]
fn full_tuple_order_and_numeric_limit_are_preserved() {
    let policy = control(Policy::SystemAllocation);
    let plain = key(b"same", None, policy);
    let two = key(b"same", Some((b"predicate", 2)), policy);
    let ten = key(b"same", Some((b"predicate", 10)), policy);
    let different = key(b"same", Some((b"predicatf", 0)), policy);
    assert!(plain < two && two < ten && ten < different);
    assert!(two == key(b"same", Some((b"predicate", 2)), policy));
}

#[test]
fn run_replacement_deduplicates_full_bodies_and_moves_canonical_and_authored_orders() {
    let policy = control(Policy::SystemAllocation);
    let mut store = RetainedFactStore::new(policy).unwrap();
    for value in (0..96).rev() {
        let locator = format!("fact-{value:03}");
        store
            .insert_for_test(
                key(locator.as_bytes(), None, policy),
                Body {
                    scalar: value,
                    roles: 1,
                },
                policy,
                merge,
            )
            .unwrap();
    }
    // Boundary and interior keys are in directory runs, not the author chunk.
    for value in [0, 70, 95] {
        let locator = format!("fact-{value:03}");
        store
            .insert_for_test(
                key(locator.as_bytes(), None, policy),
                Body {
                    scalar: value,
                    roles: 2,
                },
                policy,
                merge,
            )
            .unwrap();
    }
    let canonical = store.finish_for_test(policy).unwrap();
    assert_eq!(canonical.len(), 96);
    for (expected, record) in canonical.iter().enumerate() {
        assert_eq!(
            (*record.key_for_test()).locator_for_test(),
            format!("fact-{expected:03}").as_bytes()
        );
        assert_eq!((*record.value_for_test()).scalar, expected as u64);
        assert_eq!(
            (*record.value_for_test()).roles,
            if [0, 70, 95].contains(&expected) {
                3
            } else {
                1
            }
        );
    }
    let authored = restore_authored_order(canonical, policy).unwrap();
    for (ordinal, record) in authored.iter().enumerate() {
        assert_eq!((*record.ordinal_for_test()), ordinal);
        assert_eq!((*record.value_for_test()).scalar, (95 - ordinal) as u64);
    }
}

#[test]
fn conflicting_body_refuses_the_entire_attempt() {
    let policy = control(Policy::SystemAllocation);
    for conflicting_value in [0, 10, 63] {
        let mut conflict = RetainedFactStore::new(policy).unwrap();
        for value in 0..64 {
            let locator = format!("fact-{value:03}");
            conflict
                .insert_for_test(
                    key(locator.as_bytes(), None, policy),
                    Body {
                        scalar: value,
                        roles: 1,
                    },
                    policy,
                    merge,
                )
                .unwrap();
        }
        let locator = format!("fact-{conflicting_value:03}");
        assert!(matches!(
            conflict.insert_for_test(
                key(locator.as_bytes(), None, policy),
                Body {
                    scalar: 999,
                    roles: 1
                },
                policy,
                merge
            ),
            Err(StoreDenial::ConflictingBody)
        ));
        assert!(matches!(
            conflict.finish_for_test(policy),
            Err(StoreDenial::ConflictingBody)
        ));
    }
}

pub fn control<'scope, 'authority>(
    policy: Policy<'scope, 'authority>,
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'scope, 'authority>{
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(policy, None)
}

#[test]
fn sealed_canonical_seed_moves_into_merge_without_cloning_values() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<worth_execution::ExecutionArray<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::RetainedFact<Body>>>();
    let policy = control(Policy::SystemAllocation);
    let mut original = RetainedFactStore::new(policy).unwrap();
    for value in 0..65 {
        let locator = format!("fact-{value:03}");
        original
            .insert_for_test(
                key(locator.as_bytes(), None, policy),
                Body {
                    scalar: value,
                    roles: 1,
                },
                policy,
                merge,
            )
            .unwrap();
    }
    let sealed = original.finish_for_test(policy).unwrap();
    let mut merged = RetainedFactStore::from_sorted_records_for_test(sealed, policy).unwrap();
    merged
        .insert_for_test(
            key(b"fact-064", None, policy),
            Body {
                scalar: 64,
                roles: 2,
            },
            policy,
            merge,
        )
        .unwrap();
    merged
        .insert_for_test(
            key(b"fact-065", None, policy),
            Body {
                scalar: 65,
                roles: 1,
            },
            policy,
            merge,
        )
        .unwrap();
    let sealed = merged.finish_for_test(policy).unwrap();
    assert_eq!(sealed.len(), 66);
    for (index, record) in sealed.iter().enumerate() {
        assert_eq!((*record.value_for_test()).scalar, index as u64);
        assert_eq!((*record.ordinal_for_test()), index);
        assert_eq!(
            (*record.value_for_test()).roles,
            if index == 64 { 3 } else { 1 }
        );
    }
}

#[test]
fn physical_content_allocation_is_allocation_denial_without_fabricated_sdk_cause() {
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind as Kind;
    let denied =
        StoreDenial::PhysicalAllocation.into_attempt_denial_for_test("retained content scratch");
    assert_eq!(denied.kind(), Kind::AllocationDenied);
    assert!(denied.allocation_denial().is_none());
    assert!(denied.source_retention_interruption().is_none());
    let overflow =
        StoreDenial::Representability.into_attempt_denial_for_test("retained content scratch");
    assert_eq!(overflow.kind(), Kind::RetainedSourceStorageDenied);
}

#[path = "custody/mod.rs"]
pub mod custody;
