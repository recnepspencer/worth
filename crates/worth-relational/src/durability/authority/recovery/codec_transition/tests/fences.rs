//! Real protected source, relation and invariant consequence refusals.
use super::fixture::*;
use crate::identity::data::{KindId, PartitionId};
use crate::schema::data::*;
use crate::transactions::data::*;
use crate::validation::data::*;
use worth_foundational::facade::*;

#[test]
fn changed_authored_source_codec_cannot_acquire_derived_retirement_permission() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution, true);
    let mut target_schema = registry(2);
    let mut source = target_schema.entity_kinds.remove(&KindId(1)).unwrap();
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
        target_schema.register_entity_kind(source).unwrap(),
        Vec::new(),
    );
    let (_, mut recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap();
    let before = protected_payload(&target, world.authored);
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
            crate::mvcc::RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
    );
    assert_eq!(protected_payload(&target, world.authored), before);
    assert!(selected.is_current());
    assert!(!recovered.codec_transition_retires(world.authored));
}

#[test]
fn actual_incident_relation_refuses_retirement_without_cascading_protected_payload() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_with(with_relation(registry(1)), Vec::new());
    let world = world_with_installation(execution, runtime, receipt, true);
    let mut transaction = world
        .runtime
        .begin_branch_transaction(
            &basis(&world.runtime),
            crate::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("protected relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(4),
                    client_key: crate::symbols::data::ClientKey::raw("uses-output"),
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
    let (_, mut recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap();
    let selected = basis(&target);
    let before = protected_payload(&target, world.authored);
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
            crate::mvcc::RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .unwrap_err();
    assert!(denied.detail().contains("incident relation"), "{denied:?}");
    assert!(selected.is_current());
    assert_eq!(protected_payload(&target, world.authored), before);
    let current = basis(&target);
    let partition = current
        .inner
        .root
        .partition_state(PartitionId::main())
        .unwrap();
    assert_eq!(
        partition
            .relation_arena
            .occupied_slots_iter()
            .filter(|slot| partition.relation_arena.get_slot(*slot).unwrap().is_live())
            .count(),
        1
    );
}

#[test]
fn affected_installed_custom_invariant_refuses_before_restoring_source_into_target() {
    let request = crate::test_execution_policy::serial_request();
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
    let denied = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap_err();
    let crate::durability::data::RelationalRecoveryStop::Durability(denied) = denied else {
        panic!("custom consequence requires a schema refusal")
    };
    assert!(denied.detail.contains("custom invariant"), "{denied:?}");
    assert_eq!(before, target.runtime_instance_id());
    assert!(target.history().latest_commit().is_none());
}

fn with_relation(registry: RelationalSchemaRegistry) -> RelationalSchemaRegistry {
    registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(4),
            kind_name: "uses-derived".into(),
            schema_id: SchemaId("native-codec-world".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: crate::config::data::CrossContextPolicy::Forbid,
            cascade_delete_policy: crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
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
