//! D11 for the identities only output publication records: a required
//! output's producer basis and a codec retirement's source basis. Two
//! independent worlds, each with its own source and recovered target runtime,
//! publish byte-identical artifacts.
mod destination_admission;
mod fences;
mod fixture;
mod index_adoption;
mod output_correspondence;
mod publication_custody;
mod publication_fixture;
mod related;
mod retained_records;
use fixture::*;
use serde::de::IgnoredAny;
use worth_relational::facade::{
    self,
    durability::{
        RecoveredCheckpointTransitionError, RecoveredRelationalRuntimeAuthority,
        RelationalCanonicalCommitArtifact,
    },
    mvcc::{RelationalCancellationSource, RelationalOperationControl},
    runtime::{RelationalInitialSchemaInstallationReceipt, RelationalRuntime},
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

fn artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(|artifact| {
            artifact
                .expect("retained commits encode")
                .as_bytes()
                .to_vec()
        })
        .collect()
}

/// Run the genuine codec transition from `world` into a recovered v2 target.
fn transitioned(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> RelationalRuntime {
    let (mut target, _receipt) = installed(2);
    let (_, mut recovered) = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .expect("the source checkpoint restores for transition");
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .expect("the recovered source issues its publication owner");
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
        .expect("the transition begins");
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .expect("the transition prepares");
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .expect("the transition commits");
    let (_, recovered) = committed.into_parts();
    assert!(recovered.codec_transition_retires(world.output));
    target
}

#[test]
fn output_origin_and_codec_retirement_artifacts_are_runtime_neutral() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let first = world(execution, true);
    let second = world(execution, true);
    assert_ne!(
        first.runtime.runtime_instance_id(),
        second.runtime.runtime_instance_id()
    );
    let published = artifacts(execution, &first.runtime);
    assert!(published.iter().any(|frame| {
        output_fields(execution, frame)
            .required_output_origin
            .is_some()
    }));
    assert!(
        published == artifacts(execution, &second.runtime),
        "output publication artifacts differ"
    );

    let first_target = transitioned(execution, &first);
    let second_target = transitioned(execution, &second);
    let first_artifacts = artifacts(execution, &first_target);
    assert!(first_artifacts.iter().any(|frame| {
        !output_fields(execution, frame)
            .required_output_codec_retirements
            .is_empty()
    }));
    assert!(
        first_artifacts == artifacts(execution, &second_target),
        "codec retirement artifacts differ"
    );
}

// Re-admit the original full frame, then inspect its serialized output fields
// without a private envelope representation or an additional test door.
#[derive(serde::Deserialize)]
struct OutputFields {
    required_output_origin: Option<IgnoredAny>,
    required_output_codec_retirements: Vec<IgnoredAny>,
}

fn output_fields(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    frame: &[u8],
) -> OutputFields {
    let admitted = RelationalCanonicalCommitArtifact::readmit(frame, execution).unwrap();
    let exact = rmp_serde::to_vec_named(admitted.envelope()).unwrap();
    rmp_serde::from_slice(&exact).unwrap()
}

fn recovered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> (
    RelationalRuntime,
    RecoveredRelationalRuntimeAuthority,
    RelationalInitialSchemaInstallationReceipt,
    facade::durability::RelationalNativeCheckpoint,
) {
    let (mut target, receipt) = installed(2);
    let saved = capture(execution, &world.runtime);
    let (_, recovered) = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &saved)
        .unwrap();
    assert!(recovered.codec_transition_pending());
    (target, recovered, receipt, saved)
}

#[test]
fn ordinary_and_equal_ordinary_overwritten_rows_cannot_acquire_producer_retirement_permission() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for genuine in [false, true] {
        let world = world(execution, genuine);
        if genuine {
            rewrite_output(execution, &world);
        }
        let (mut target, mut recovered, _, prior_saved) = recovered(execution, &world);
        let prior = locators(&prior_saved);
        let before = capture_carrying(execution, &target, &prior);
        let source = protected_payload(execution, &target, world.authored);
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
            facade::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
        );
        assert_eq!(
            source,
            protected_payload(execution, &target, world.authored)
        );
        assert_eq!(before, capture_carrying(execution, &target, &prior));
        assert!(recovered.codec_transition_pending());
    }
}

#[test]
fn interrupted_native_codec_publication_returns_the_exact_source_custody_for_retry() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let (mut target, mut recovered, _, _) = recovered(execution, &world);
    let selected = basis(&target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
    let cancel = RelationalCancellationSource::new();
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
    assert!(target
        .owner_component_services()
        .basis_port()
        .compare_current_exact(&selected)
        .is_ok());
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        protected_payload(execution, &world.runtime, world.authored)
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
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    world
        .runtime
        .index_authority()
        .register(facade::indexes::DerivedIndexDefinition {
            index_id: facade::indexes::DerivedIndexId(18),
            name: "unsupported derived ordering".into(),
            branch_scoped: true,
            unique_entity_kind: None,
            kind: facade::indexes::DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: facade::identity::KindId(99),
                parent_endpoint: facade::indexes::RelatedEntityEndpoint::SourceParent,
                child_kind: facade::identity::KindId(3),
                ordering: vec![facade::indexes::RelatedEntityOrderingField::new(
                    locator("derived"),
                    facade::indexes::RelatedEntityOrderingDirection::Ascending,
                )],
            },
        })
        .expect("index fixture has identity capacity");
    let (mut target, _) = installed(2);
    let initial_id = target.runtime_instance_id();
    let initial_schema = target.config().schema.registry;
    let denied = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap_err();
    assert!(
        matches!(denied,facade::durability::RelationalRecoveryStop::Durability(error) if error.class==facade::durability::RecoveryFailureClass::SchemaMismatch)
    );
    assert_eq!(target.runtime_instance_id(), initial_id);
    assert_eq!(target.config().schema.registry, initial_schema);
    assert!(target.history().historical_latest_commit().is_none());
}

mod activation_admission;

mod source_head_admission;

mod stale_codec_revision;
