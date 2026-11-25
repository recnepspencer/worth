//! Real protected source, relation and invariant consequence refusals.
use super::fixture::*;
use runtime::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
};
use worth_foundational::facade::*;
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    durability::RelationalRecoveryStop,
    identity::{KindId, PartitionId},
    mvcc::{RelationalOperationControl, RelationalTransactionIntent},
    runtime::{self, RelationalSchemaTransitionAdmissionDenialKind},
    schema::*,
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[test]
fn changed_authored_source_codec_cannot_acquire_derived_retirement_permission() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let target_schema = registry(2);
    let mut source = target_schema
        .entity_registration(KindId(1))
        .unwrap()
        .clone();
    let previous = &source.aspect_contract_declarations.aspects[0].contract;
    let AspectShape::Struct(shape) = previous.shape() else {
        panic!("fixture source is struct")
    };
    source.aspect_contract_declarations.aspects[0].contract = AspectContract::struct_aspect(
        previous.key().clone(),
        previous.identity(),
        AspectContractRevision(2),
        StructAspectShape::new(
            shape
                .fields()
                .iter()
                .cloned()
                .map(|field| field.with_codec(CanonicalDigestId::new([31; 32]))),
        )
        .unwrap(),
    );
    let (mut target, _) = installed_with(
        RelationalSchemaRegistry::new()
            .register_entity_kind(source)
            .unwrap()
            .register_entity_kind(
                target_schema
                    .entity_registration(KindId(2))
                    .unwrap()
                    .clone(),
            )
            .unwrap()
            .register_entity_kind(
                target_schema
                    .entity_registration(KindId(3))
                    .unwrap()
                    .clone(),
            )
            .unwrap(),
        Vec::new(),
    );
    let (_, mut recovered) = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap();
    let before = protected_payload(execution, &target, world.authored);
    let selected = basis(&target);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
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
        RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
    );
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        before
    );
    assert!(target
        .owner_component_services()
        .basis_port()
        .compare_current_exact(&selected)
        .is_ok());
    assert!(!recovered.codec_transition_retires(world.authored));
}

#[test]
fn actual_incident_relation_refuses_retirement_without_cascading_protected_payload() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_with(with_relation(registry(1)), Vec::new());
    let world = world_with_installation(execution, runtime, receipt, true);
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &basis(&world.runtime),
            RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("protected relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(4),
                    client_key: ClientKey::raw("uses-output"),
                    source: EntityReference::Existing(world.authored),
                    target: EntityReference::Existing(world.output),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let (mut target, _) = installed_with(with_relation(registry(2)), Vec::new());
    let (_, mut recovered) = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap();
    let selected = basis(&target);
    let before = protected_payload(execution, &target, world.authored);
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .unwrap();
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
    assert!(denied.detail().contains("incident relation"), "{denied:?}");
    assert!(target
        .owner_component_services()
        .basis_port()
        .compare_current_exact(&selected)
        .is_ok());
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        before
    );
    let snapshot = target
        .snapshots()
        .snapshot_for_observation(&basis(&target).observation())
        .unwrap();
    let view = target
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(
        view.relations()
            .iter()
            .filter(
                |relation| relation.relation_id.partition_id == PartitionId::main()
                    && relation.lifecycle
                        == worth_relational::facade::storage::RecordLifecycleState::Live
            )
            .count(),
        1
    );
    target.snapshots().release_snapshot(&snapshot).unwrap();
}

#[test]
fn affected_installed_custom_invariant_refuses_before_restoring_source_into_target() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_with(
        registry(1),
        vec![CustomInvariantRegistration::new(OutputInvariant).unwrap()],
    );
    let world = world_with_installation(execution, runtime, receipt, true);
    let (mut target, _) = installed_with(
        registry(2),
        vec![CustomInvariantRegistration::new(OutputInvariant).unwrap()],
    );
    let before = target.runtime_instance_id();
    let denied = crate::relational::relational_certification::cold_residency::codec_neutrality::fixture::restore_codec_transition(&mut target, execution, &capture(execution, &world.runtime))
        .unwrap_err();
    let RelationalRecoveryStop::Durability(denied) = denied else {
        panic!("custom consequence requires a schema refusal")
    };
    assert!(denied.detail.contains("custom invariant"), "{denied:?}");
    assert_eq!(before, target.runtime_instance_id());
    assert!(target.history().historical_latest_commit().is_none());
}

fn with_relation(registry: RelationalSchemaRegistry) -> RelationalSchemaRegistry {
    registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(4),
            kind_name: "uses-derived".into(),
            schema_id: SchemaId("native-codec-world".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap()
}

struct OutputInvariant;
impl CustomInvariantRule for OutputInvariant {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("codec.output.invariant"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Output invariant".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1024).unwrap()),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(3)],
                    affected_entity_kinds: vec![KindId(3)],
                    ..Default::default()
                },
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}
