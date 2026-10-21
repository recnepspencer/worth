//! A borrowed new Fact never imports somebody else's heap admission.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::facts::payload::FactStorage,
    application_contribution::with_bootstrap_advancement,
};
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::{
    facade::{AspectKey, AspectValue, FieldKey, InternedString},
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    transactions::planned_single_field_locator,
};

#[test]
fn new_heap_copy_refuses_before_copy_and_funding_survives_views_until_final_drop() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 24), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let borrowed = [super::super::Fact::Field {
            entity_id: EntityId::new(PartitionId::main(), 1, 1),
            kind: KindId::new(7),
            locator: planned_single_field_locator(
                AspectKey::new("details").unwrap(),
                FieldKey::new("payload").unwrap(),
            ),
            value: AspectValue::String(InternedString::from("g".repeat(1 << 20))),
        }];
        let original = FactPayload::copy_funded(&borrowed, control).unwrap();
        let baseline = lease.charged_memory_bytes();
        let quote = super::super::chunk::deep_clone_bytes(&borrowed[0]).unwrap();
        let held = lease
            .reserve_memory(
                lease
                    .available_memory_bytes()
                    .expect("this fixture supplies a finite memory ceiling")
                    - quote
                    + 1,
            )
            .unwrap();
        let Err(StoreDenial::Allocation(refusal)) = FactPayload::copy_funded(&borrowed, control)
        else {
            panic!("missing new deep grant must refuse")
        };
        assert_eq!(refusal.requested_payload_bytes(), Some(quote));
        assert_eq!(original.get(0), Some(&borrowed[0]));
        drop(held);
        assert_eq!(lease.charged_memory_bytes(), baseline);
        let result = FactPayload::copy_funded(&borrowed, control).unwrap();
        let FactStorage::Native(chunk) = &result.storage else {
            panic!("one bounded leaf")
        };
        assert!(chunk._copied_heaps.as_ref().unwrap().bytes() >= quote);
        let weak = std::sync::Arc::downgrade(chunk);
        let view = result.slice(0..1);
        let clone = view.clone();
        drop(result);
        drop(view);
        assert!(weak.upgrade().is_some());
        assert_eq!(clone.get(0), Some(&borrowed[0]));
        drop(clone);
        assert!(weak.upgrade().is_none());
        assert_eq!(lease.charged_memory_bytes(), baseline);
        let cancel = worth_execution::CancellationSource::new();
        let child = lease.controlled_child(cancel.token(), None);
        cancel.cancel();
        assert!(FactPayload::copy_funded(
            &borrowed,
            StorageControl::new(ExecutionAllocationPolicy::Execution(&child), None)
        )
        .is_err());
        assert_eq!(lease.charged_memory_bytes(), baseline);
        assert_eq!(
            FactPayload::copy_funded(&borrowed, control).unwrap().get(0),
            original.get(0)
        );
    })
    .unwrap();
}
