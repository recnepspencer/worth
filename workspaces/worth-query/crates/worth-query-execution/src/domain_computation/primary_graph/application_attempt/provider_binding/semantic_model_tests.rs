mod authoritative_truth;
mod model;
mod world;

use self::model::observe;
use self::world::mixed_effect_world;
use super::{prepare_provider_attempt, WorthQueryApplicationRealizedEffect};
use crate::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedRelation;
use crate::domain_computation::primary_graph::application_attempt::{
    effect_program::WorthQueryCandidateValidatorWorkAdmission,
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationObservedFact,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};

#[test]
fn mixed_effects_lower_to_the_exact_independent_semantic_model() {
    let world = mixed_effect_world();
    let prepared = prepare_provider_attempt(
        Vec::new(),
        world.facts,
        world.effects,
        world.retained_bytes,
        world.retained_bytes,
        None,
        None,
        WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
        Default::default(),
    )
    .expect("complete mixed effect basis should lower");

    assert_eq!(observe(prepared), world.expected);
}

#[test]
fn alternate_effect_insertion_preserves_each_exact_association_and_order() {
    let world = mixed_effect_world();
    let prepared = prepare_provider_attempt(
        Vec::new(),
        world.facts,
        world.alternate_effects,
        world.retained_bytes,
        world.retained_bytes,
        None,
        None,
        WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
        Default::default(),
    )
    .expect("complete mixed effect basis should lower");
    assert_eq!(observe(prepared), world.alternate_expected);
}

#[test]
fn two_relation_deletes_from_one_adjacency_share_one_provisional_retirement() {
    let from = EntityId::new(PartitionId::main(), 1, 0);
    let first_to = EntityId::new(PartitionId::main(), 2, 0);
    let second_to = EntityId::new(PartitionId::main(), 3, 0);
    let first_relation = RelationId::new(PartitionId::main(), 4, 0);
    let second_relation = RelationId::new(PartitionId::main(), 5, 0);
    let relation_kind = KindId::new(31);
    let facts = vec![WorthQueryApplicationObservedFact::Adjacency {
        relation_kind,
        anchor: from,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: 2,
        relations: vec![
            WorthQueryApplicationObservedRelation {
                relation_id: first_relation,
                from,
                to: first_to,
            },
            WorthQueryApplicationObservedRelation {
                relation_id: second_relation,
                from,
                to: second_to,
            },
        ],
    }];
    let effects = vec![
        WorthQueryApplicationRealizedEffect::DeleteRelation {
            relation_id: first_relation,
        },
        WorthQueryApplicationRealizedEffect::DeleteRelation {
            relation_id: second_relation,
        },
    ];
    let prepared = prepare_provider_attempt(
        Vec::new(),
        facts,
        effects,
        0,
        0,
        None,
        None,
        WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
        Default::default(),
    )
    .expect("both relation deletes are authorized by the observed adjacency");

    assert_eq!(prepared.effects.expected_steps().len(), 1);
}
