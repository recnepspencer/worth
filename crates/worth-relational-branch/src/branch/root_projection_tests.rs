//! Projection borrows the selected live or retired storage image.
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use super::{RelationalBranchRoot, RelationalBranchRootIdentityIssuer};
use crate::branch::basis_admission_tests::installed_history;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::history::retention::{RelationalRetainedHistoricalRoot, VisibilitySnapshotBasis};
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::runtime::state::subsystems::services::{
    instrumentation::RuntimeInstrumentation, symbol_table::RuntimeSymbolTable,
};
use crate::schema::data::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::substrate::EntityExtra;
use crate::transactions::data::primitives::RecordRef;
use crate::visibility::materialization::read_records::projection::{
    creation::ProjectionInputs, EntityProjectionRecord, ProjectionAspectScope,
    VisibilityProjectionView,
};
use crate::visibility::snapshot_states::{HistoricalVisibilityBasis, SnapshotStateBasis};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    validate_aspect_value, AspectBinding, AspectContract, AspectContractRevision, AspectIdentity,
    AspectKey, AspectValue, AuthoritativeRecordAspectPatch, ContractValidationInput, FieldKey,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn aspect_key(name: &str) -> AspectKey {
    AspectKey::new(name).unwrap()
}

#[test]
fn projection_preserves_identity_kind_and_lifecycle_across_exact_and_historical_bases() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (entity, before, after) = selected_images(execution, "live");
    for (before, after) in before.into_iter().zip(after) {
        assert_projection_lifecycle(execution, &before, &after, entity);
    }
}

fn assert_projection_lifecycle(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &VisibilityProjectionView,
    after: &VisibilityProjectionView,
    entity: EntityId,
) {
    let scope = ProjectionAspectScope::whole_aspects([aspect_key("name")]);
    let value =
        |record: EntityProjectionRecord<'_>| record.aspect_value(&aspect_key("name")).cloned();
    assert_eq!(
        before
            .entity_record_with_projection_scope(execution, entity, scope.clone(), value)
            .expect("read request remains live"),
        Some(AspectValue::String("live".into()))
    );
    assert!(after
        .entity_record_with_projection_scope(execution, entity, scope.clone(), value)
        .expect("read request remains live")
        .is_none());
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot_value(),
        entity.generation_value() + 1,
    );
    assert!(before
        .entity_record_with_projection_scope(execution, stale, scope.clone(), value)
        .expect("read request remains live")
        .is_none());
    assert_eq!(
        before
            .entity_record_of_expected_kind_with_projection_scope(
                execution,
                entity,
                KindId(2),
                scope.clone(),
                value
            )
            .expect("read request remains live"),
        Err(KindId(1))
    );
    let wildcard = EntityId::new(entity.partition_id, entity.local_slot_value(), 0);
    assert_eq!(
        before
            .entity_record_with_projection_scope(execution, wildcard, scope, |record| Some(
                record.entity_id()
            ))
            .expect("read request remains live"),
        Some(entity)
    );
}

#[test]
fn wrong_kind_refusal_precedes_projection_mask_admission() {
    let request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (entity, before, _) = selected_images(execution, "wrong-kind");
    for view in before {
        let called = std::cell::Cell::new(false);
        let result = view.entity_record_of_expected_kind_with_projection_scope(
            execution,
            entity,
            KindId(2),
            ProjectionAspectScope::fields(
                aspect_key("undeclared"),
                [FieldKey::new("field").unwrap()],
            ),
            |_| {
                called.set(true);
                Some(())
            },
        );
        assert_eq!(result, Ok(Err(KindId(1))));
        assert!(
            !called.get(),
            "a wrong-kind record must not reach the projection callback"
        );
    }
}

fn selected_images(
    execution: &ExecutionRequest<'_, '_>,
    value: &str,
) -> (
    EntityId,
    [VisibilityProjectionView; 2],
    [VisibilityProjectionView; 2],
) {
    let contract = AspectContract::scalar(
        aspect_key("name"),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "projected-entity".into(),
            schema_id: SchemaId("projection".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: FieldKey::new("name").unwrap(),
                    },
                    contract: contract.clone(),
                },
            ]),
        })
        .unwrap();
    let TransitionOutcome::Success(admitted) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::String(value.into())),
    ) else {
        panic!("the name is in the declared scalar contract");
    };
    let TransitionOutcome::Success(aspects) =
        AuthoritativeRecordAspectPatch::whole_aspect([admitted], [])
    else {
        panic!("one admitted scalar has no duplicate bindings");
    };
    let TransitionOutcome::Success(Some(payload)) = aspects.apply_to_optional(None) else {
        panic!("the created row has its initial name");
    };
    let partition_id = PartitionId::main();
    let (mut partition, entities, _) =
        crate::storage::overlay::small_relation_partition::relation_partition_for_test(
            partition_id,
            1,
            &[],
        );
    let entity = entities[0];
    partition
        .entity_arena
        .apply_extra_update_for_test(
            entity.slot_index(),
            EntityExtra::from_unbudgeted_parts(
                None,
                None,
                Some(payload.into_parts().into_parts().0),
            ),
            VersionId(1),
        )
        .unwrap();
    let branch = BranchId("main".into());
    let history = installed_history(1);
    history.install_recovered_sequence_floor(3, 3);
    let binding = history
        .branch_cell(&branch)
        .unwrap()
        .head_retention()
        .binding()
        .unwrap();
    let issuer = RelationalBranchRootIdentityIssuer::default();
    let symbols = RuntimeSymbolTable::default();
    let instrumentation = RuntimeInstrumentation::default();
    let installed = ProjectionInputs {
        runtime_identity: 1,
        history: &history,
        instrumentation: &instrumentation,
        symbols: &symbols,
        empty_history_schema: &registry,
    };
    let capture = |version: VersionId,
                   partition: crate::storage::overlay::PartitionState,
                   previous: Option<&Arc<RelationalBranchRoot>>| {
        let receipt = RelationalCommitReceipt {
            commit_id: CommitId(version.0),
            version_id: version,
            branch_id: branch.clone(),
            parents: previous
                .iter()
                .filter_map(|root| root.commit_id())
                .collect(),
        };
        let mut envelope =
            crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_at(
                RecordRef::Entity(entity),
                entity,
                receipt,
            );
        envelope.body_mut_for_test().schema_authority = registry.authority_snapshot();
        let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
        journal.entity_slots.insert(entity.slot_index());
        let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
            &BTreeMap::from([(partition_id, (partition.clone(), journal))]),
        );
        RelationalBranchRoot::prepare_capture(
            execution,
            &issuer,
            &BTreeMap::from([(partition_id, partition)]),
            &delta,
            previous,
            Arc::new(envelope),
            &Default::default(),
            &registry,
            &symbols,
            None,
        )
        .unwrap()
        .into_root()
    };
    let live = capture(VersionId(1), partition.clone(), None);
    partition
        .entity_arena
        .retire_record_for_test(entity.slot_index(), VersionId(2))
        .unwrap();
    let retired = capture(VersionId(2), partition, Some(&live));
    let project = |version, root: Arc<RelationalBranchRoot>| {
        let exact =
            VisibilitySnapshotBasis::from_applied_root(branch.clone(), version, root.clone());
        let retained = RelationalRetainedHistoricalRoot::acquire(&binding, root).unwrap();
        let historical = HistoricalVisibilityBasis::from_retained_interval(
            branch.clone(),
            version,
            version,
            retained,
        );
        [
            VisibilityProjectionView::new(installed, SnapshotStateBasis::Exact(exact)),
            VisibilityProjectionView::new(installed, SnapshotStateBasis::Historical(historical)),
        ]
    };
    (
        entity,
        project(VersionId(1), live),
        project(VersionId(2), retired),
    )
}
