//! A born relation names an endpoint this draft allocated only when it truly does.
use super::*;
use crate::transactions::data::{CreatedEntityRef, RelationSpec};

#[test]
fn only_births_with_an_allocated_endpoint_carry_structural_prior_absence() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let first = create_entity(execution, &runtime, "first");
    let second = create_entity(execution, &runtime, "second");
    let third = create_entity(execution, &runtime, "third");
    let moved = create_relation(execution, &runtime, first, second, "moved");
    let born = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("born"),
    };
    let link = |key: &str, source, target| {
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw(key),
            source,
            target,
            fields: crate::tests::support::relation_label_field_patch(key),
        }))
    };
    let validated = validate(
        execution,
        &runtime,
        [
            MutationIntent::Create(CreateIntent::Entity(entity_spec("born"))),
            link(
                "from-born",
                EntityReference::Created(born.clone()),
                EntityReference::Existing(second),
            ),
            link(
                "to-born",
                EntityReference::Existing(first),
                EntityReference::Created(born),
            ),
            link(
                "between-existing",
                EntityReference::Existing(third),
                EntityReference::Existing(first),
            ),
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: moved,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(second),
                    target: EntityReference::Existing(first),
                },
            )),
        ],
    );
    let footprint = validated
        .relation_mutation_footprint(execution, &[KindId(2)])
        .expect("the validated draft projects its relation lifecycle");
    let mut births = Vec::new();
    for mutation in footprint.relations() {
        if mutation.relation() == moved {
            assert!(
                mutation.before().is_some(),
                "an endpoint change is not a birth"
            );
            assert!(!mutation.has_allocated_endpoint());
            continue;
        }
        let (source, target) = mutation.after().expect("every other change is a birth");
        assert!(mutation.before().is_none());
        births.push(((source, target), mutation.has_allocated_endpoint()));
    }
    assert_eq!(births.len(), 3, "{births:?}");
    for ((source, target), allocated) in births {
        let existing = [first, second, third];
        assert_eq!(
            allocated,
            !(existing.contains(&source) && existing.contains(&target)),
            "{source:?}->{target:?}"
        );
    }
}
