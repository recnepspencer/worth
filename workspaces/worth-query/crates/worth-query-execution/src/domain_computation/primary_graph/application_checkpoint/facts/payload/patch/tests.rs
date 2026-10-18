//! Storage proof only: these ordinals never establish graph/read currentness.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::facts::NativeFactPayloadBuilder,
    application_contribution::with_bootstrap_advancement,
};
use crate::facade::primary_graph::WorthQueryExecutionObservation;
use std::sync::{Arc, Weak};
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::{
    facade::{AspectKey, AspectValue, FieldKey, InternedString},
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    transactions::planned_single_field_locator,
};

fn fact(ordinal: usize) -> Fact {
    Fact::Field {
        entity_id: EntityId::new(PartitionId::main(), ordinal as u64, 1),
        kind: KindId::new(7),
        locator: planned_single_field_locator(
            AspectKey::new("details").unwrap(),
            FieldKey::new("payload").unwrap(),
        ),
        value: AspectValue::String(InternedString::from(format!(
            "{ordinal}:{}",
            "x".repeat(1024)
        ))),
    }
}
fn native(count: usize, control: StorageControl<'_, '_>) -> FactPayload {
    let mut builder = NativeFactPayloadBuilder::allocate(count, control).unwrap();
    for ordinal in 0..count {
        builder.push(fact(ordinal)).unwrap();
    }
    builder.seal().unwrap()
}
fn leaf(payload: &FactPayload, ordinal: usize) -> &Arc<super::super::chunk::NativeFactChunk> {
    match &payload.storage {
        FactStorage::Native(chunk) => chunk,
        FactStorage::Joined { children } if ordinal < children[0].len() => {
            leaf(&children[0], ordinal)
        }
        FactStorage::Joined { children } => leaf(&children[1], ordinal - children[0].len()),
        _ => panic!("native fixture leaf"),
    }
}
fn policy() -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1 << 24, 1_000_000),
    )
}

#[test]
fn sparse_patches_copy_only_admitted_changed_chunks_and_release_displaced_backing() {
    let _observed = WorthQueryExecutionObservation::install_on_this_thread();
    with_bootstrap_advancement(policy(), |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let baseline = lease.charged_memory_bytes();
        for count in [20, 65, 515] {
            let original = native(count, control);
            let ordinal = count / 2;
            let old_chunk: Weak<_> = Arc::downgrade(leaf(&original, ordinal));
            let quote = leaf(&original, ordinal).clone_bytes.unwrap();
            observation::fact_verification_on_this_thread_for_test();
            let before = lease.charged_memory_bytes();
            let mut patch = original
                .prepare_native_patch(&im::ordset![ordinal], control)
                .unwrap();
            assert!(
                lease.charged_memory_bytes() >= before + quote,
                "deep funding exists before copying"
            );
            let mut replacement = fact(ordinal);
            if let Fact::Field { value, .. } = &mut replacement {
                *value = AspectValue::UInt64(73);
            }
            patch.replace(ordinal, &replacement, control).unwrap();
            let result = patch.seal(control).unwrap();
            let work = observation::fact_verification_on_this_thread_for_test();
            assert!(
                work.native_patch_fact_clones <= 32,
                "{count}: bounded actual deep copies: {work:?}"
            );
            assert!(
                work.native_payload_size_visits <= 32,
                "{count}: quote maintenance visits only the changed leaf"
            );
            assert!(
                work.native_patch_path_visits
                    <= 16 * (count.next_power_of_two().ilog2() as u64 + 1),
                "{count}: actual paths, not a full index constructor: {work:?}"
            );
            for position in 0..count {
                assert_eq!(
                    result.get(position),
                    if position == ordinal {
                        Some(&replacement)
                    } else {
                        original.get(position)
                    }
                );
                if position / 32 != ordinal / 32 {
                    assert!(
                        std::ptr::eq(
                            result.get(position).unwrap(),
                            original.get(position).unwrap()
                        ),
                        "untouched leaf keeps its actual backing"
                    );
                }
            }
            assert!(
                leaf(&result, ordinal)
                    ._copied_heaps
                    .as_ref()
                    .unwrap()
                    .bytes()
                    >= quote
            );
            drop(original);
            assert!(
                old_chunk.upgrade().is_none(),
                "no old-parent/history chain is retained by the replacement leaf"
            );
            let view = result.slice(ordinal..ordinal + 1);
            let clone = view.clone();
            drop(result);
            drop(view);
            assert_eq!(clone.get(0), Some(&replacement));
            drop(clone);
            assert_eq!(
                lease.charged_memory_bytes(),
                baseline,
                "all shared native and copy grants release after final holder"
            );
        }
    })
    .unwrap();
}

#[test]
fn patch_refusal_precedes_copy_and_cancelled_author_never_seals_partial_facts() {
    let _observed = WorthQueryExecutionObservation::install_on_this_thread();
    with_bootstrap_advancement(policy(), |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let original = native(65, control);
        let baseline = lease.charged_memory_bytes();
        let quote = leaf(&original, 37).clone_bytes.unwrap();
        let held = lease
            .reserve_memory(lease.available_memory_bytes() - quote + 1)
            .unwrap();
        observation::fact_verification_on_this_thread_for_test();
        assert!(original
            .prepare_native_patch(&im::ordset![37], control)
            .is_err());
        assert_eq!(
            observation::fact_verification_on_this_thread_for_test().native_patch_fact_clones,
            0,
            "no uncharged copy on underfunding"
        );
        drop(held);
        assert_eq!(lease.charged_memory_bytes(), baseline);
        let incomplete = original
            .prepare_native_patch(&im::ordset![37], control)
            .unwrap();
        assert!(matches!(
            incomplete.seal(control),
            Err(StoreDenial::ConflictingBody)
        ));
        assert_eq!(lease.charged_memory_bytes(), baseline);
        let mut cancelled = original
            .prepare_native_patch(&im::ordset![37], control)
            .unwrap();
        assert!(
            cancelled.replace(36, &fact(36), control).is_err(),
            "unplanned ordinal"
        );
        cancelled.replace(37, &fact(37), control).unwrap();
        assert!(
            cancelled.replace(37, &fact(37), control).is_err(),
            "duplicate fill"
        );
        let source = worth_execution::CancellationSource::new();
        let child = lease.controlled_child(source.token(), None);
        source.cancel();
        assert!(cancelled
            .seal(StorageControl::new(
                ExecutionAllocationPolicy::Execution(&child),
                None
            ))
            .is_err());
        assert_eq!(lease.charged_memory_bytes(), baseline);
        assert_eq!(original.get(37), Some(&fact(37)));
        let mut retry = original
            .prepare_native_patch(&im::ordset![37], control)
            .unwrap();
        retry.replace(37, &fact(73), control).unwrap();
        assert_eq!(retry.seal(control).unwrap().get(37), Some(&fact(73)));
        assert_eq!(lease.charged_memory_bytes(), baseline);
    })
    .unwrap();
}

#[test]
fn growing_replacement_is_admitted_before_copy_and_keeps_funding_until_final_view_drop() {
    let _observed = WorthQueryExecutionObservation::install_on_this_thread();
    with_bootstrap_advancement(policy(), |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let empty = lease.charged_memory_bytes();
        let mut tiny = fact(0);
        if let Fact::Field { value, .. } = &mut tiny {
            *value = AspectValue::String(InternedString::from("a"));
        }
        let mut author = NativeFactPayloadBuilder::allocate(1, control).unwrap();
        author.push(tiny.clone()).unwrap();
        let original = author.seal().unwrap();
        let old = Arc::downgrade(leaf(&original, 0));
        let mut growing = fact(0);
        if let Fact::Field { value, .. } = &mut growing {
            *value = AspectValue::String(InternedString::from("g".repeat(1 << 20)));
        }
        // This borrowed producer value supplies meaning, never transferable
        // admission. The patch must fund its OWN new deep copy.
        let quote = super::super::chunk::deep_clone_bytes(&growing).unwrap();
        assert!(quote >= 1 << 20);
        let baseline = lease.charged_memory_bytes();
        let mut patch = original
            .prepare_native_patch(&im::ordset![0], control)
            .unwrap();
        let prepared = lease.charged_memory_bytes();
        let held = lease
            .reserve_memory(lease.available_memory_bytes() - quote + 1)
            .unwrap();
        observation::fact_verification_on_this_thread_for_test();
        let Err(StoreDenial::Allocation(denial)) = patch.replace(0, &growing, control) else {
            panic!("growing replacement must refuse without its own deep-copy grant")
        };
        assert!(matches!(
            denial.kind(),
            worth_execution::ExecutionAllocationDenialKind::Lease(
                worth_execution::LeaseDenial::MemoryExhausted(_)
            )
        ));
        assert_eq!(denial.requested_payload_bytes(), Some(quote));
        assert_eq!(
            observation::fact_verification_on_this_thread_for_test().native_patch_fact_clones,
            0
        );
        assert_eq!(original.len(), 1);
        assert_eq!(original.get(0), Some(&tiny));
        assert!(Arc::ptr_eq(leaf(&original, 0), &old.upgrade().unwrap()));
        drop(held);
        assert_eq!(
            lease.charged_memory_bytes(),
            prepared,
            "refused copy retains no grant"
        );
        patch.replace(0, &growing, control).unwrap();
        assert_eq!(lease.charged_memory_bytes(), prepared + quote);
        let result = patch.seal(control).unwrap();
        assert_eq!(result.get(0), Some(&growing));
        assert_eq!(original.get(0), Some(&tiny));
        assert!(lease.charged_memory_bytes() >= baseline + quote);
        let published = Arc::downgrade(leaf(&result, 0));
        drop(original);
        assert!(
            old.upgrade().is_none(),
            "replacement has no history-parent chain"
        );
        let funded = lease.charged_memory_bytes();
        let view = result.slice(0..1);
        let clone = view.clone();
        drop(result);
        drop(view);
        assert_eq!(clone.get(0), Some(&growing));
        assert_eq!(
            lease.charged_memory_bytes(),
            funded,
            "view retains the actual new grant"
        );
        assert!(funded >= empty + quote);
        drop(clone);
        assert!(published.upgrade().is_none());
        assert_eq!(
            lease.charged_memory_bytes(),
            empty,
            "last holder releases new and old-copy grants"
        );
    })
    .unwrap();
}

#[test]
fn decoded_or_opaque_original_facts_cannot_be_relabelled_by_native_patch() {
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
    let original: FactPayload = Arc::<[Fact]>::from([fact(0)]).into();
    let funding = super::super::CodecPayloadReservation::admit(control, 4096).unwrap();
    let decoded = FactPayload::decoded(vec![fact(0)], funding);
    for payload in [original, decoded] {
        assert!(payload
            .prepare_native_patch(&im::ordset![0], control)
            .is_err());
        let unchanged = payload
            .prepare_native_patch(&im::ordset![], control)
            .unwrap()
            .seal(control)
            .unwrap();
        assert!(
            payload.same_backing(&unchanged),
            "sharing grants no native reinterpretation"
        );
    }
}
