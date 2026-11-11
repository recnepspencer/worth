//! Existing committed Native index evidence from the actual performed Home step.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    indexes::{
        BoundedEntityFieldLookupOutcome, DerivedIndexDefinition, DerivedIndexEntries,
        DerivedIndexId,
    },
    runtime::RelationalRuntime,
};

pub(super) fn observe(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    old_definition: &DerivedIndexDefinition,
    old_id: DerivedIndexId,
    target_definition: &DerivedIndexDefinition,
    target_id: DerivedIndexId,
) -> BoundedEntityFieldLookupOutcome {
    assert_eq!(
        native
            .index_access()
            .matching_definition(old_definition)
            .unwrap()
            .index_id,
        old_id
    );
    assert_eq!(
        native
            .index_access()
            .matching_definition(target_definition)
            .unwrap()
            .index_id,
        target_id
    );
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    let generation = native
        .index_access()
        .read_published_generation_for_observation(execution, target_id, &observation)
        .unwrap()
        .unwrap();
    let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
        panic!("actual B field generation");
    };
    assert_eq!(
        entries
            .values()
            .map(|entities| entities.len())
            .sum::<usize>(),
        1
    );
    let receipt = native.history().historical_latest_commit().unwrap();
    let envelope = native
        .replay()
        .canonical_commit_envelope(execution, receipt.commit_id)
        .unwrap()
        .unwrap();
    let wire: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
    let definitions = wire["index_definition_delta"]["definitions"]
        .as_array()
        .unwrap();
    assert!(definitions
        .iter()
        .any(|definition| definition["index_id"] == target_id.0
            && definition["name"] == "target-label"));
    let view = native
        .read_truth()
        .project_observation(&observation)
        .unwrap();
    let worth_relational::facade::indexes::DerivedIndexKind::EntityField { field_locator } =
        &target_definition.kind
    else {
        unreachable!();
    };
    let value = worth_foundational::facade::AspectValue::String("target-value".into());
    let lookup = native
        .index_access()
        .execute_bounded_entity_field_lookup(
            worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup::new(
                &view,
                target_id,
                worth_relational::facade::identity::KindId(2000),
                field_locator,
                &value,
                10,
            ),
            worth_relational::facade::indexes::BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert_eq!(lookup.candidate_entity_ids().len(), 1);
    lookup
}
