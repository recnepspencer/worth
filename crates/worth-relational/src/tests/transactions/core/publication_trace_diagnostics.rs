use crate::facade::diagnostics::{
    DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticsProfile,
};
use crate::facade::transactions::MutationIntent;
use crate::tests::support::*;
use worth_foundational::facade::AspectKey;

#[test]
fn commit_publication_exposes_aspect_evaluation_and_emission_traces() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let result = create_entity_outcome(&runtime, "traced");
    let evaluation_traces = result.aspect_evaluation_traces();
    let emission_traces = result.aspect_emission_traces();
    let patch_vs_truth = assert_patch_truth_invariants(&result);

    assert_eq!(evaluation_traces.len(), 1);
    assert_eq!(emission_traces.len(), 1);
    assert_eq!(
        evaluation_traces[0].target,
        RecordRef::Entity(changed_entities(&result)[0])
    );
    assert_eq!(evaluation_traces[0].kind_id, KindId(1));
    assert_eq!(
        evaluation_traces[0].structural_change,
        RecordStructuralChange::Created
    );
    assert_eq!(
        evaluation_traces[0].changed_aspects,
        ordered_aspect_keys([
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("name").unwrap(),
        ])
    );
    assert_eq!(evaluation_traces[0].binding_rows.len(), 2);
    assert_eq!(emission_traces[0].target, evaluation_traces[0].target);
    assert_eq!(emission_traces[0].patch_position, result.patch_position());
    assert_eq!(emission_traces[0].patch_record_index, 0);
    assert_eq!(
        emission_traces[0].changed_aspects,
        evaluation_traces[0].changed_aspects
    );
    assert!(patch_vs_truth.exact_match);
    assert_eq!(patch_vs_truth.records_checked, 1);
    assert_eq!(result.aspect_tag_accuracy_report().records_checked, 1);
    assert_eq!(
        result.aspect_tag_accuracy_report().correctly_tagged_records,
        1
    );
}

#[test]
fn detailed_trace_profile_emits_commit_side_aspect_trace_diagnostics() {
    let diagnostics = RelationalDiagnosticsProfile {
        detailed_traces_enabled: true,
        ..RelationalDiagnosticsProfile::default()
    };
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .diagnostics(diagnostics)
        .build();
    let result = create_entity_outcome(&runtime, "diagnostic-traced");

    assert!(result.diagnostics().iter().any(|artifact| {
        artifact.scope == DiagnosticsScope::Transaction
            && artifact.kind == DiagnosticsArtifactKind::DetailedTrace
            && artifact
                .entries
                .iter()
                .any(|entry| entry.code == DiagnosticCode::AspectEvaluationTraced)
    }));
    assert!(result.diagnostics().iter().any(|artifact| {
        artifact.scope == DiagnosticsScope::PatchPublication
            && artifact.kind == DiagnosticsArtifactKind::DetailedTrace
            && artifact
                .entries
                .iter()
                .any(|entry| entry.code == DiagnosticCode::AspectEmissionTraced)
    }));
}

#[test]
fn aspect_evaluation_trace_retains_unchanged_bindings_for_auditability() {
    let fixture = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_field_aspect(
                crate::tests::support::aspect_key("status"),
                crate::tests::support::field_key("status"),
            ),
            lifecycle_aspect(),
        ],
        relation_aspects: vec![relation_source_aspect(), relation_target_aspect()],
        ..AspectSchemaFixture::default()
    };
    let runtime = fixture.build_runtime();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("create").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("row"),
                fields: crate::tests::support::string_aspect_field_patch([
                    (
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "before",
                    ),
                    (
                        crate::tests::support::aspect_key("status"),
                        crate::tests::support::field_key("status"),
                        "stable",
                    ),
                ]),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let created = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let entity = changed_entities(&created)[0];

    let mut update_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    update_txn
        .push_batch(
            WorkerIntentBatch::new("update-name-only").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: crate::tests::support::string_aspect_field_patch([
                        (
                            crate::tests::support::aspect_key("name"),
                            crate::tests::support::field_key("name"),
                            "after",
                        ),
                        (
                            crate::tests::support::aspect_key("status"),
                            crate::tests::support::field_key("status"),
                            "stable",
                        ),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let result = update_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let trace = &result.aspect_evaluation_traces()[0];
    let status_key = AspectKey::new("status").unwrap();
    let status_row = trace
        .binding_rows
        .iter()
        .find(|row| row.aspect_key == status_key)
        .expect("status aspect row");

    assert_eq!(trace.binding_rows.len(), 3);
    assert!(!status_row.changed);
    assert!(!trace
        .changed_aspects
        .iter()
        .any(|aspect| aspect == &status_key));
}
