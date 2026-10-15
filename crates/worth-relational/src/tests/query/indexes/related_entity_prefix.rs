//! Real leased prefix navigation, complete publication and last-owner backing release.
use super::*;
use crate::facade::identity::EntityId;
use crate::facade::indexes::{
    BoundedRelatedEntityOrderedLookupRequest as Ordered,
    BoundedRelatedEntityPrefixDenial as Denial, BoundedRelatedEntityPrefixLookupRequest as Prefix,
    DerivedIndexEntries,
};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn related_entity_prefix_leased_seek_refusal_and_retained_page() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let parent = changed_entities(&create_entity_outcome(&setup, &runtime, "parent"))[0];
    let mut expected = Vec::new();
    for ordinal in 0..34 {
        let name = if ordinal < 2 {
            "alpha".to_owned()
        } else {
            format!("unrelated-{ordinal}")
        };
        let child = changed_entities(&create_entity_outcome(&setup, &runtime, &name))[0];
        create_relation_outcome(&setup, &runtime, parent, child, "owns");
        if ordinal < 2 {
            expected.push(child)
        }
    }
    expected.sort();
    let index = super::related_entity_ordered_lookup::register_related_name_index(&runtime);
    let built = runtime.index_authority().build_for_commit(
        &setup,
        DerivedIndexBuildRequest {
            source_commit_id: runtime.history().latest_commit().unwrap().commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(built.failed_indexes.is_empty());
    let snapshot = runtime.visibility_authority().snapshot();
    let pin = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[index.index_id], &setup)
        .unwrap();
    let values = [string_aspect_value("alpha")];
    let ordered = || {
        Ordered::new(snapshot.clone(), index.index_id, parent, KindId(1), None, 1)
            .unwrap()
            .expect_generation(built.generations[0].generation_id)
    };
    let lease = |memory, work| {
        test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, memory, work),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap()
    };
    let work_denied = lease(1024 * 1024, 1);
    let stopped = pin
        .execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &values, None),
            &ExecutionRequest::leased(&work_denied),
        )
        .unwrap_err();
    assert!(matches!(
        stopped,
        Denial::Work(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::WorkCeiling
        ))
    ));
    assert_eq!(work_denied.charged_memory_bytes(), 0);
    let memory_denied = lease(0, 1_000_000);
    assert!(matches!(
        pin.execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &values, None),
            &ExecutionRequest::leased(&memory_denied)
        ),
        Err(Denial::Work(worth_execution::WorkCeilingDenial::Admission(
            _
        ))) | Err(Denial::Allocation(_))
    ));
    let cancelled = worth_execution::CancellationSource::new();
    let cancelled_lease = test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 1024 * 1024, 1_000_000),
            ),
            deadline: None,
            cancellation: cancelled.token(),
        })
        .unwrap();
    cancelled.cancel();
    assert!(matches!(
        pin.execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &values, None),
            &ExecutionRequest::leased(&cancelled_lease)
        ),
        Err(Denial::Work(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        )))
    ));
    let funded = lease(1024 * 1024, 1_000_000);
    let request = ExecutionRequest::leased(&funded);
    let first = pin
        .execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &values, None),
            &request,
        )
        .unwrap();
    assert_eq!(first.child_entity_ids(), &expected[..1]);
    assert!(
        first.seek_comparison_count() > 0 && first.seek_comparison_count() < 32,
        "binary seeks, not unrelated-parent filtering"
    );
    assert!(first.charged_payload_bytes().unwrap() > 0);
    assert!(
        funded.charged_memory_bytes() > 0,
        "page and boundary reservations survive scope exit"
    );
    let second = pin
        .execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &values, first.next_boundary()),
            &request,
        )
        .unwrap();
    assert_eq!(second.child_entity_ids(), &expected[1..]);
    assert!(second.next_boundary().is_none());
    let foreign = [string_aspect_value("unrelated-2")];
    assert!(matches!(
        pin.execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &foreign, first.next_boundary()),
            &request
        ),
        Err(Denial::ForeignBoundary)
    ));
    let wrong_type = [worth_foundational::facade::AspectValue::UInt64(7)];
    assert!(matches!(
        pin.execute_bounded_related_entity_prefix_lookup(
            Prefix::new(ordered(), &wrong_type, None),
            &request
        ),
        Err(Denial::InvalidPrefix)
    ));
    // Observe native framework occupancy, rather than copying its layout formula.
    // Both the measurement and lookup run one nested scope under the active child.
    let parent_lease = lease(4 * 1024 * 1024, 1_000_000);
    let child_request = |memory| LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, memory, 1_000_000),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let child = parent_lease.child(child_request(1024 * 1024)).unwrap();
    let ceiling = worth_execution::ExecutionWorkCeiling::new(1_000_000);
    let (framework, _) = ceiling
        .run(&child, || {
            ceiling
                .run(&child, || child.charged_memory_bytes())
                .unwrap()
                .0
        })
        .unwrap();
    assert!(framework > 0);
    let basis_bytes = snapshot.branch_id().0.len() as u64;
    let id_bytes = std::mem::size_of::<EntityId>() as u64;
    assert!(basis_bytes > 0);
    // Separately exhaust projection backing, IDs, and continuation backing AFTER entry.
    for (stage, available) in [
        basis_bytes - 1,
        basis_bytes + id_bytes - 1,
        2 * basis_bytes + id_bytes - 1,
    ]
    .into_iter()
    .enumerate()
    {
        let tight = parent_lease
            .child(child_request(framework + available))
            .unwrap();
        let (refused, _) = ceiling
            .run(&tight, || {
                pin.execute_bounded_related_entity_prefix_lookup(
                    Prefix::new(ordered(), &values, None),
                    &ExecutionRequest::leased(&parent_lease),
                )
            })
            .unwrap();
        match &refused {
            Err(Denial::Memory(cause)) if stage != 1 => {
                assert_eq!(cause.requested, basis_bytes);
                assert_eq!(
                    cause.level,
                    worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
                );
            }
            Err(Denial::Allocation(cause)) if stage == 1 => {
                assert_eq!(cause.requested_payload_bytes(), Some(id_bytes));
                assert!(
                    matches!(cause.kind(), worth_execution::ExecutionAllocationDenialKind::Lease(
                    worth_execution::LeaseDenial::MemoryExhausted(cause)
                ) if cause.level == worth_execution::MemoryLimitLevel::Policy { ancestor: 0 })
                );
            }
            _ => panic!(
                "framework admitted; backing stage {stage} must hit the child ceiling: {refused:?}"
            ),
        }
        assert_eq!(tight.charged_memory_bytes(), 0);
        assert_eq!(parent_lease.charged_memory_bytes(), 0);
    }
    let (child_page, _) = ceiling
        .run(&child, || {
            pin.execute_bounded_related_entity_prefix_lookup(
                Prefix::new(ordered(), &values, None),
                &ExecutionRequest::leased(&parent_lease),
            )
        })
        .unwrap();
    let child_page = child_page.unwrap();
    assert_eq!(child_page.child_entity_ids(), &expected[..1]);
    assert!(child.charged_memory_bytes() > child_page.charged_payload_bytes().unwrap());
    assert_eq!(
        parent_lease.charged_memory_bytes(),
        child.charged_memory_bytes()
    );
    drop(child_page);
    assert_eq!(child.charged_memory_bytes(), 0);
    assert_eq!(parent_lease.charged_memory_bytes(), 0);

    drop((first, second));
    assert_eq!(
        funded.charged_memory_bytes(),
        0,
        "last payload and boundary owner releases original payer"
    );
    // Empty prefix means complete range, but still requires every stored ordering value.
    let complete = pin
        .execute_bounded_related_entity_prefix_lookup(Prefix::new(ordered(), &[], None), &request)
        .unwrap();
    assert_eq!(complete.child_entity_ids(), &expected[..1]);
    drop(complete);
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::RelatedEntityOrdering(entries) = &mut generation.entries
            else {
                panic!("related ordering expected");
            };
            let rows = entries.get_mut(&parent).unwrap();
            let original = &rows[0];
            let truncated = crate::indexes::data::RelatedEntityOrderingEntry::new(
                vec![],
                original.child_entity_id(),
                original.relation_id(),
            );
            rows[0] = truncated;
        });
    let corrupt_pin = runtime
        .read_truth()
        .pin_read_capability(&snapshot, &[index.index_id], &setup)
        .unwrap();
    let refused = corrupt_pin
        .execute_bounded_related_entity_prefix_lookup(Prefix::new(ordered(), &[], None), &request)
        .unwrap_err();
    assert!(matches!(refused, Denial::Lookup(cause)
        if cause.kind() == crate::facade::indexes::BoundedRelatedEntityOrderedLookupDenialKind::CorruptIndexEntries));
    assert_eq!(funded.charged_memory_bytes(), 0);
}
