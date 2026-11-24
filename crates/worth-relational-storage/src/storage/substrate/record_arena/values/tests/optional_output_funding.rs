//! Actual optional payload results retain no discarded State roots.
use super::*;

#[test]
fn cleared_entity_and_relation_extras_keep_only_their_fixed_original_bills() {
    let (contract, input) = patch("original");
    let TransitionOutcome::Success(clear) = AuthoritativeRecordAspectPatch::whole_aspect(
        std::iter::empty::<ContractValidatedAspectArtifact>(),
        [contract],
    ) else {
        panic!("actual whole-aspect clear")
    };
    let source_budget = SerialMemoryBudget::new(None);
    let source_host = request(&source_budget);
    let source_request = ExecutionRequest::serial(&source_host);
    let entity = EntityExtra::apply_payload(&source_request, None, &input).unwrap();
    let relation = RelationExtra::apply_payload(&source_request, None, &input).unwrap();
    let source_bill = source_budget.charged_memory_bytes();
    assert!(source_bill > 0);
    let receiving = SerialMemoryBudget::new(None);
    let receiving_host = request(&receiving);
    let receiving_request = ExecutionRequest::serial(&receiving_host);
    let cleared_entity = EntityExtra::apply_payload(
        &receiving_request,
        entity.authoritative_aspect_state(),
        &clear,
    )
    .unwrap();
    let cleared_relation = RelationExtra::apply_payload(
        &receiving_request,
        relation.authoritative_aspect_state(),
        &clear,
    )
    .unwrap();
    assert!(cleared_entity.authoritative_aspect_state().is_none());
    assert!(cleared_relation.authoritative_aspect_state().is_none());
    assert_eq!(source_budget.charged_memory_bytes(), source_bill);
    assert_eq!(
        receiving.charged_memory_bytes(),
        StorageAllocation::<EntityExtra>::layout_bytes()
            + StorageAllocation::<RelationExtra>::layout_bytes()
    );
    drop((entity, relation));
    assert_eq!(source_budget.charged_memory_bytes(), 0);
    drop(cleared_entity);
    assert_eq!(
        receiving.charged_memory_bytes(),
        StorageAllocation::<RelationExtra>::layout_bytes()
    );
    drop(cleared_relation);
    assert_eq!(receiving.charged_memory_bytes(), 0);
}
