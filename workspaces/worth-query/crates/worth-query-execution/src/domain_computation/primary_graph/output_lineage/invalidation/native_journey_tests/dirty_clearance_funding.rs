//! Fresh carriage metadata has its own payer and survives every cloned holder.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use worth_execution::{
    CancellationToken, ExecutionAllocationPolicy, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub(super) fn assert_carriage_funding(
    owner: &crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner,
    basis: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    changes: &crate::domain_computation::primary_graph::output_lineage::invalidation::ComputationChanges,
    original: &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::MIN,
        charged_memory_bytes: None,
    })
    .unwrap();
    let ask = |bytes| LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, bytes, Some(1_000_000)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let fresh = OrdSet::unit(0);
    let zero = authority.request_lease(ask(Some(0))).unwrap();
    let control = StorageControl::new(ExecutionAllocationPolicy::Execution(&zero), None);
    assert!(
        changes
            .carry(original, 1, &fresh, snapshot, control)
            .is_err(),
        "zero payer refuses fresh metadata before transfer"
    );
    let lease = authority.request_lease(ask(Some(8))).unwrap();
    let control = StorageControl::new(ExecutionAllocationPolicy::Execution(&lease), None);
    let baseline = lease.charged_memory_bytes();
    let carry = changes
        .carry(original, 1, &fresh, snapshot, control)
        .unwrap()
        .unwrap();
    assert_eq!(lease.charged_memory_bytes(), baseline + 8);
    let held = carry.clone();
    drop(carry);
    assert_eq!(
        lease.charged_memory_bytes(),
        baseline + 8,
        "clone retains original metadata grant"
    );
    drop(held);
    assert_eq!(
        lease.charged_memory_bytes(),
        baseline,
        "last drop releases original grant"
    );
    assert!(
        changes
            .carry(original, 1, &OrdSet::new(), snapshot, control)
            .unwrap()
            .is_some(),
        "refused attempt preserves original source custody"
    );
    // This is a capacity oracle only; these literal facts issue no read authority.
    use crate::domain_computation::primary_graph::{
        application_attempt::WorthQueryApplicationObservedFact as Fact,
        application_checkpoint::facts::FactPayload,
    };
    use worth_foundational::facade::{AspectKey, FieldKey};
    let locator = worth_relational::facade::transactions::planned_single_field_locator(
        AspectKey::new("capacity").unwrap(),
        FieldKey::new("value").unwrap(),
    );
    let facts = FactPayload::from(Arc::<[Fact]>::from([Fact::SourceFieldRevision {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            17,
            1,
        ),
        locator,
        native_revision: None,
    }]));
    let mut cold_work = owner.edit_admission();
    let cold = super::super::super::fact_retention::reserve(
        &facts,
        basis,
        &owner.resources,
        &mut cold_work,
    )
    .unwrap();
    let payload = super::super::super::fact_retention::fact_payload_bytes(
        &facts[0],
        &mut owner.edit_admission(),
    )
    .unwrap();
    assert!(
        payload > 0,
        "the oracle contains an actual owned locator payload"
    );
    let mut cached_work = owner.edit_admission();
    let cached = super::super::super::fact_retention::reserve(
        &facts,
        basis,
        &owner.resources,
        &mut cached_work,
    )
    .unwrap();
    assert_eq!(
        cached.bytes(),
        cold.bytes(),
        "cached bound preserves the exact cold capacity contract"
    );
    assert!(
        cached_work.charged_work() < cold_work.charged_work(),
        "the cached path executes no old payload walk"
    );
}
