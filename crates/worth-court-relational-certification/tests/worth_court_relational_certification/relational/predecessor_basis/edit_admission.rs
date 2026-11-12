//! Native edit admission authenticates the current full-generation source.
use super::*;
use worth_relational::facade::{
    identity::EntityId,
    publication::AuthoredRetirementRefusal as Cause,
    transactions::{RecordRef, TransactionCommitError},
};

#[test]
fn ordinary_edit_custody_requires_a_live_source_in_its_exact_current_basis() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution);
    let runtime = &world.runtime;
    let installed = &world.installed;
    let source = world.source;
    let basis = world.basis();
    let admitted = runtime
        .readmit_edit_coupled_owners(execution, installed, &basis, source)
        .unwrap();
    assert!(
        admitted.requires_coupled_assembly(),
        "Initial outputs still need dispatch"
    );
    assert_eq!(admitted.required_owner_count(), 0);
    admitted.require_independent().unwrap();
    drop(admitted);
    let before = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let missing = EntityId::new(
        source.partition_id,
        source.local_slot_value(),
        source.generation_value() + 1,
    );
    let denied = runtime
        .readmit_edit_coupled_owners(execution, installed, &basis, missing)
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = denied else {
        panic!("{denied:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::CoupledSourceNotLive)
    );
    assert_eq!(error.context.affected_records, [RecordRef::Entity(missing)]);
    assert_eq!(
        runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap(),
        before
    );

    world.change_source(execution, "other-edit");
    let before = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let denied = runtime
        .readmit_edit_coupled_owners(execution, installed, &basis, source)
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = denied else {
        panic!("{denied:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(Cause::ForeignCoupledOwner)
    );
    assert_eq!(error.context.affected_records, [RecordRef::Entity(source)]);
    assert_eq!(
        runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap(),
        before
    );
}
