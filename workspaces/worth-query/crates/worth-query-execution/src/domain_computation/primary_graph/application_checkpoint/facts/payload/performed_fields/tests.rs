//! Readonly capture severs the parent body and retains its own original payer.
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
fn performed_field_copy_severs_parent_and_retains_original_request_funding() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 24), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("actual original payer")
        };
        let baseline = lease.charged_memory_bytes();
        let control = StorageControl::new(execution.allocation_policy(), None);
        let entity = EntityId::new(PartitionId::main(), 1, 7);
        let fields = [
            Fact::SourceEntity { entity_id: entity },
            Fact::Field {
                entity_id: entity,
                kind: KindId::new(7),
                locator: planned_single_field_locator(
                    AspectKey::new("data").unwrap(),
                    FieldKey::new("input").unwrap(),
                ),
                value: AspectValue::String(InternedString::from("x".repeat(1 << 16))),
            },
        ];
        let original = FactPayload::copy_funded(&fields, control).unwrap();
        let FactStorage::Native(backing) = &original.storage else {
            panic!("original leaf")
        };
        let parent = std::sync::Arc::downgrade(backing);
        let captured = FactPayload::copy_performed_field_reads(&original, control).unwrap();
        assert!(!captured.same_backing(&original));
        assert_eq!(captured.len(), 1);
        assert_eq!(captured.get(0), Some(&fields[1]));
        drop(original);
        assert!(
            parent.upgrade().is_none(),
            "capture cannot pin a hidden parent body"
        );
        assert!(lease.charged_memory_bytes() > baseline);
        let retained = captured.clone();
        drop(captured);
        assert_eq!(retained.get(0), Some(&fields[1]));
        let cancel = worth_execution::CancellationSource::new();
        let child = lease.controlled_child(cancel.token(), None);
        cancel.cancel();
        assert!(FactPayload::copy_performed_field_reads(
            &retained,
            StorageControl::new(ExecutionAllocationPolicy::Execution(&child), None)
        )
        .is_err());
        drop(retained);
        assert_eq!(lease.charged_memory_bytes(), baseline);
    })
    .unwrap();
}
