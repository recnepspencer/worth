//! Borrowed payload and installed field support for an exact relation.
use super::pinned_read_publication::snapshot_for_main;
use super::proposal_records::{
    create_entity_with_fields, create_relation_in_partition, name_field_patch,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{config::CascadeDeletePolicy, identity::PartitionId};
use worth_testkit_relational::{
    aspect_storage::AspectSchemaFixture,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn pinned_relation_aspect_reads_borrow_scalar_and_authored_coordinates() {
    use worth_foundational::facade::{
        AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey,
    };

    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let partition = PartitionId::main();
    let runtime = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_runtime();
    let source =
        create_entity_with_fields(&execution, &runtime, "source", name_field_patch("source"));
    let target =
        create_entity_with_fields(&execution, &runtime, "target", name_field_patch("target"));
    let relation = create_relation_in_partition(
        &execution,
        &runtime,
        source,
        target,
        "stored-label",
        partition,
    );
    let pin = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &execution)
        .unwrap();
    let label = AspectKey::new("label").unwrap();
    let key = FieldKey::new("label").unwrap();
    let projection = pin.project_read();

    assert_eq!(
        pin.read_relation_aspect(&execution, relation, &label, |value| {
            value.is_some_and(|value| match value.view() {
                ContractValidatedAspectValueView::Scalar(AspectValue::String(text)) => {
                    text.raw_text() == Some("stored-label")
                }
                _ => false,
            })
        })
        .unwrap(),
        Some(true)
    );
    assert_eq!(pin.read_relation_aspect_field(&execution, relation, &label, &key, |value| {
        matches!(value, Some(AspectValue::String(text)) if text.raw_text() == Some("stored-label"))
    }).unwrap(), Some(true));
    assert_eq!(
        projection
            .relation_aspect_support_fields(&execution, &projection, relation, &label)
            .unwrap()
            .collect::<Vec<_>>(),
        vec![&key]
    );
    assert_eq!(
        projection
            .relation_aspect_presence_scalar_field(&execution, &projection, relation, &label)
            .unwrap(),
        Some(&key)
    );
    assert!(pin
        .read_relation_aspect_version(&execution, relation, &label)
        .unwrap()
        .flatten()
        .is_some());
    assert_eq!(
        pin.read_relation_aspect_version(&execution, relation, &AspectKey::new("absent").unwrap())
            .unwrap(),
        Some(None)
    );
}
