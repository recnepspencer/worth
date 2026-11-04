//! Independent selected-field oracle, separate from content-directory resolution.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContentRefId, ContractValidatedAspectValueView, FieldKey,
};
use worth_relational::facade::{identity::EntityId, runtime::RelationalRuntime};

pub(super) fn assert_selected(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    expected: ContentRefId,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let read = runtime
        .read_truth()
        .read_observation(execution, &basis.observation())
        .unwrap()
        .unwrap();
    let record = read.get_entity(entity).unwrap();
    let state = record.authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Struct(fields) = state
        .get(&AspectKey::new("samples").unwrap())
        .unwrap()
        .view()
    else {
        panic!("the selected content aspect must be a struct");
    };
    for field in ["left", "right"] {
        assert_eq!(
            fields.get(&FieldKey::new(field).unwrap()),
            Some(&AspectValue::ContentRef(expected))
        );
    }
}
