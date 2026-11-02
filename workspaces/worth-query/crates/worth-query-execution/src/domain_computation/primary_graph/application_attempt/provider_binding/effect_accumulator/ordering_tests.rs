//! Authored position is independent of record identity sorting and composition.
use super::*;
use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectKey, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::{
    identity::{EntityId, PartitionId, RelationId},
    transactions::{
        ApplyEntityAspectPatchIntent, DeleteEntityIntent, DeleteRelationIntent,
        EntityMutationIntent, RelationMutationIntent,
    },
};
worth_query_declaration::worth_query_structured_value_binding!(Payload for u64 {
    identity: "effect-order.payload.v1"
});

fn clear(name: &str) -> PortableAspectPatchOperation {
    PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::new(
            AspectKey::new(name).unwrap(),
            AspectIdentity(1),
            AspectContractRevision(1),
        ),
    }
}
fn patch(id: EntityId, name: &str) -> WorthQueryLoweredProviderEffect {
    WorthQueryLoweredProviderEffect::EntityAspectPatch {
        entity_id: id,
        operations: vec![clear(name)],
        steps: vec![],
    }
}
fn mutation(intent: MutationIntent) -> WorthQueryLoweredProviderEffect {
    WorthQueryLoweredProviderEffect::Mutation {
        intent,
        steps: vec![],
    }
}
fn delete(id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id: id,
    }))
}
fn composed(id: EntityId, names: &[&str]) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
        ApplyEntityAspectPatchIntent {
            entity_id: id,
            aspect_patch: PortableRecordAspectPatch::new(names.iter().map(|name| clear(name))),
        },
    ))
}
#[test]
fn first_patch_positions_preserve_plain_mutations_and_ordered_emissions() {
    let a = EntityId::new(PartitionId::main(), 9, 2);
    let b = EntityId::new(PartitionId::main(), 9, 1);
    let single = EntityId::new(PartitionId::new(7), 9, 1);
    let deleted = EntityId::new(PartitionId::main(), 20, 1);
    let edge = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: RelationId::new(PartitionId::main(), 31, 2),
    }));
    let effects = vec![
        patch(a, "a-first"),
        mutation(delete(deleted)),
        WorthQueryLoweredProviderEffect::Emission(WorthQueryApplicationEmission::new::<Payload>(
            "first", 11,
        )),
        mutation(edge.clone()),
        patch(b, "b-first"),
        patch(single, "single"),
        WorthQueryLoweredProviderEffect::ObservedNoop { steps: vec![] },
        patch(a, "a-last"),
        WorthQueryLoweredProviderEffect::Emission(WorthQueryApplicationEmission::new::<Payload>(
            "last", 22,
        )),
        patch(b, "b-last"),
        mutation(delete(single)),
    ];
    let (intents, emissions, _) = materialize_commit_projections(effects).unwrap();
    // Full literal intent order AND payloads; a single patch and a delete on
    // that record both survive for Native's duplicate-target refusal.
    assert_eq!(
        intents,
        vec![
            composed(a, &["a-first", "a-last"]),
            delete(deleted),
            edge,
            composed(b, &["b-first", "b-last"]),
            composed(single, &["single"]),
            delete(single),
        ]
    );
    assert_eq!(emissions.len(), 2);
    assert_eq!(
        (emissions[0].effect(), emissions[0].payload::<u64>()),
        ("first", Some(&11))
    );
    assert_eq!(
        (emissions[1].effect(), emissions[1].payload::<u64>()),
        ("last", Some(&22))
    );
}

#[test]
fn delete_and_composed_patch_remain_distinct_in_either_authored_order() {
    let id = EntityId::new(PartitionId::new(4), 8, 3);
    for delete_first in [false, true] {
        let mut effects = vec![patch(id, "first"), patch(id, "second")];
        let at = if delete_first { 0 } else { 1 };
        effects.insert(at, mutation(delete(id)));
        let (intents, _, _) = materialize_commit_projections(effects).unwrap();
        let expected = if delete_first {
            vec![delete(id), composed(id, &["first", "second"])]
        } else {
            vec![composed(id, &["first", "second"]), delete(id)]
        };
        assert_eq!(intents, expected);
    }
}
