//! Native same-carrier adoption preserves raw authority and replaces index bytes.
mod activation_admission;
mod destination_admission;
mod fences;
mod fixture;
mod publication_custody;
mod related;
mod settlement;
mod source_head_admission;
use crate::durability::authority::{
    RecoveredCheckpointTransitionError, RecoveredRelationalRuntimeAuthority,
};
use crate::indexes::data::DerivedIndexEntries;
use crate::mvcc::RelationalOperationControl;
use crate::runtime::RelationalRuntime;
use fixture::*;

fn recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> (
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    crate::runtime::RelationalInitialSchemaInstallationReceipt,
) {
    let (mut target, receipt) = installed(2);
    let (_, recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap();
    assert!(recovered.codec_transition_pending());
    (target, recovered, receipt)
}

#[test]
fn genuine_native_codec_adoption_preserves_raw_source_and_rebuilds_target_equality_index() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let source_before = protected_payload(&world.runtime, world.authored);
    let source_carrier = world
        .runtime
        .config()
        .schema
        .registry
        .entity_kinds
        .get(&crate::identity::data::KindId(1))
        .unwrap()
        .clone();
    assert!(!source_before.is_empty());
    let old_generation = world
        .runtime
        .indexes
        .latest_generation(INDEX, Some(basis(&world.runtime).descriptor().branch_id()))
        .unwrap();
    assert!(
        matches!(&old_generation.entries,DerivedIndexEntries::EntityField(entries) if !entries.is_empty())
    );
    let (mut target, mut recovered, receipt) = recovered(execution, &world);
    assert_eq!(protected_payload(&target, world.authored), source_before);
    assert_eq!(
        target
            .config()
            .schema
            .registry
            .entity_kinds
            .get(&crate::identity::data::KindId(1)),
        Some(&source_carrier)
    );
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let selected = basis(&target);
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap();
    let (commit, recovered) = committed.into_parts();
    assert!(!recovered.codec_transition_pending());
    assert!(recovered.codec_transition_retires(world.output));
    assert!(!recovered.codec_transition_retires(world.authored));
    let target_receipt = target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(protected_payload(&target, world.authored), source_before);
    assert_eq!(
        target
            .config()
            .schema
            .registry
            .entity_kinds
            .get(&crate::identity::data::KindId(1)),
        Some(&source_carrier)
    );
    let current = basis(&target);
    let output = current
        .inner
        .root
        .partition_state(world.output.partition_id)
        .unwrap()
        .entity_arena
        .get(&world.output)
        .unwrap();
    assert!(!output.is_live());
    let target_generation = target
        .indexes
        .latest_generation(INDEX, Some(current.descriptor().branch_id()))
        .unwrap();
    assert_eq!(target_generation.source_commit_id, commit.commit_id);
    assert_ne!(
        target_generation.generation_id,
        old_generation.generation_id
    );
    assert!(
        matches!(&target_generation.entries,DerivedIndexEntries::EntityField(entries) if entries.is_empty())
    );
    recovered.admit_basis(current).unwrap();
    let (producer, installed) = target_receipt.issue_application_publication_owner();
    let regenerated = create(
        execution,
        &target,
        3,
        "derived-v2",
        r#"["new",41]"#,
        Some((
            &installed,
            producer.admit_required_output("native.fixture.producer", [10; 32]),
        )),
    );
    build_index(execution, &target);
    let rebuilt = target
        .indexes
        .latest_generation(INDEX, Some(basis(&target).descriptor().branch_id()))
        .unwrap();
    assert!(
        matches!(&rebuilt.entries,DerivedIndexEntries::EntityField(entries)
        if entries.len()==1 && entries.iter().any(|(_,ids)| ids.iter().any(|id| *id == regenerated)))
    );
    assert_eq!(protected_payload(&target, world.authored), source_before);
    let saved = capture(&target);
    let (mut reopened, receipt) = fixture::installed(2);
    reopened
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &saved)
        .unwrap();
    reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_eq!(protected_payload(&reopened, world.authored), source_before);
    assert_eq!(
        protected_payload(&reopened, regenerated),
        protected_payload(&target, regenerated)
    );
    let old = basis(&reopened);
    let saved_basis = basis(&target);
    let before = saved_basis.reference().target().as_basis().unwrap();
    let after = old.reference().target().as_basis().unwrap();
    assert_ne!(after.runtime_instance_id(), before.runtime_instance_id());
    assert_eq!(after.selected_commit_id(), before.selected_commit_id());
    assert_eq!(after.version_id(), before.version_id());
    assert_eq!(after.parent_commit_ids(), before.parent_commit_ids());
    assert_eq!(after.roots().truth_root(), before.roots().truth_root());
    assert_eq!(after.roots().schema_root(), before.roots().schema_root());
    assert!(!old
        .inner
        .root
        .partition_state(world.output.partition_id)
        .unwrap()
        .entity_arena
        .get(&world.output)
        .unwrap()
        .is_live());
    assert_eq!(
        protected_payload(&reopened, world.output),
        protected_payload(&target, world.output)
    );
    let reopened_index = reopened
        .indexes
        .latest_generation(INDEX, Some(old.descriptor().branch_id()))
        .unwrap();
    assert_eq!(reopened_index.entries, rebuilt.entries);
    assert_eq!(reopened_index.source_commit_id, rebuilt.source_commit_id);
}

#[test]
fn ordinary_and_equal_ordinary_overwritten_rows_cannot_acquire_producer_retirement_permission() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for genuine in [false, true] {
        let world = world(execution, genuine);
        if genuine {
            rewrite_output(execution, &world);
        }
        let (mut target, mut recovered, _) = recovered(execution, &world);
        let before = capture(&target);
        let source = protected_payload(&target, world.authored);
        let (owner, installed) = target
            .durability_recovery()
            .issue_recovered_codec_source_publication_owner(execution, &recovered)
            .unwrap();
        let selected = basis(&target);
        let proof = activation_proof(
            execution,
            &mut target,
            &recovered,
            &selected,
            &owner,
            world.activation,
        );
        let denied = target
            .durability_recovery()
            .begin_recovered_codec_transition(
                execution,
                &mut recovered,
                &selected,
                RelationalOperationControl::uninterrupted(),
                &installed,
                proof,
                activation(world.activation),
            )
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
        );
        assert_eq!(source, protected_payload(&target, world.authored));
        assert_eq!(before, capture(&target));
        assert!(recovered.codec_transition_pending());
    }
}

#[test]
fn interrupted_native_codec_publication_returns_the_exact_source_custody_for_retry() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let (mut target, mut recovered, _) = recovered(execution, &world);
    let selected = basis(&target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let cancel = crate::runtime::RelationalCancellationSource::new();
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            cancel.token().into(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    cancel.cancel();
    let refused = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap_err();
    let RecoveredCheckpointTransitionError::Refused(refused) = refused else {
        panic!("interruption must refuse before effects")
    };
    let mut recovered = refused.into_authority();
    assert!(recovered.codec_transition_pending());
    assert!(selected.is_current());
    assert_eq!(
        protected_payload(&target, world.authored),
        protected_payload(&world.runtime, world.authored)
    );
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap();
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let successor = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap();
    successor
        .into_parts()
        .1
        .admit_basis(basis(&target))
        .unwrap();
}

#[test]
fn unsupported_ordering_index_refuses_codec_restore_without_replacing_the_target_runtime() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    world
        .runtime
        .index_authority()
        .register(crate::indexes::data::DerivedIndexDefinition {
            index_id: crate::indexes::data::DerivedIndexId(18),
            name: "unsupported derived ordering".into(),
            branch_scoped: true,
            kind: crate::indexes::data::DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: crate::identity::data::KindId(99),
                parent_endpoint: crate::indexes::data::RelatedEntityEndpoint::SourceParent,
                child_kind: crate::identity::data::KindId(3),
                ordering: vec![crate::indexes::data::RelatedEntityOrderingField::new(
                    locator("derived"),
                    crate::indexes::data::RelatedEntityOrderingDirection::Ascending,
                )],
            },
        });
    let (mut target, _) = installed(2);
    let initial_id = target.runtime_instance_id();
    let initial_schema = target.config().schema.registry;
    let denied = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap_err();
    assert!(
        matches!(denied,crate::durability::data::RelationalRecoveryStop::Durability(error) if error.class==crate::durability::data::RecoveryFailureClass::SchemaMismatch)
    );
    assert_eq!(target.runtime_instance_id(), initial_id);
    assert_eq!(target.config().schema.registry, initial_schema);
    assert!(target.history().latest_commit().is_none());
}
