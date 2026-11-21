//! Historical workflow authority is separate from authored semantic dependencies.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_checkpoint::facts::NativeFactPayloadBuilder,
};
use worth_relational::facade::identity::PartitionId;

#[test]
fn correction_keeps_authored_overlap_and_suffix_but_excludes_appended_authority() {
    let control = StorageControl::new(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    );
    let fact = |slot| Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), slot, 1),
    };
    let mut builder = NativeFactPayloadBuilder::allocate(4, control).unwrap();
    // 0 is an authored read also observed by authority: native dedup leaves it
    // in the prefix. 1 and 2 were appended by authority; 3 is a later native read.
    for slot in 0..4 {
        builder.push(fact(slot)).unwrap();
    }
    let facts = builder.seal().unwrap();
    let semantic = semantic_facts(&facts, 1..3, control).unwrap();
    assert_eq!(
        semantic.iter().cloned().collect::<Vec<_>>(),
        [fact(0), fact(3)]
    );
    assert_eq!(
        facts.len(),
        4,
        "ordinary commit still validates every source fact"
    );
    assert!(semantic_facts(&facts, 2..2, control)
        .unwrap()
        .same_backing(&facts));
    assert!(matches!(
        semantic_facts(&facts, 1..5, control),
        Err(StoreDenial::ConflictingBody)
    ));
}
