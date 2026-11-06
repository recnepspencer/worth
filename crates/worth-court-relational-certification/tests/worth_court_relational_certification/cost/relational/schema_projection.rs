//! Read admission must never build a durable schema-authority projection.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::KindId,
    runtime::RelationalRuntimeApi,
    schema::{EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId},
};

#[test]
fn pinned_snapshot_and_view_admission_build_no_schema_projection() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "read.entity".into(),
            schema_id: SchemaId("read".into()),
            schema_version_id: SchemaVersionId(7),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let (_, positive) = RelationalSchemaRegistry::count_authority_projections_for_test(|| {
        registry.authority_snapshot()
    });
    assert!(
        positive > 0,
        "recorder must observe genuine projection construction"
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let (_, projections) = RelationalSchemaRegistry::count_authority_projections_for_test(|| {
        assert_eq!(
            runtime.read_truth().snapshot_schema_version(&snapshot),
            Some(SchemaVersionId(7))
        );
        let pin = runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[], &execution)
            .unwrap();
        let view = runtime.read_truth().project_snapshot(&snapshot).unwrap();
        let view_pin = runtime
            .read_truth()
            .pin_view_read_capability(&view, &[], &execution)
            .unwrap();
        assert_eq!(pin.read_context().schema_version, SchemaVersionId(7));
        assert_eq!(view_pin.read_context().schema_version, SchemaVersionId(7));
    });
    assert_eq!(
        projections, 0,
        "read admission reconstructed schema authority"
    );
}
