use super::*;
use crate::config::data::AdjacencyBackend;

fn inputs() -> RecordedCanonicalInputs {
    RecordedCanonicalInputs {
        result_image: None,
        prestate_schema: [1; 32],
        result_schema: [2; 32],
        mutation: MutationConfig {
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            adjacency_policy: AdjacencyPolicy {
                backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
                small_degree_inline_capacity: 4,
            },
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        },
    }
}
#[test]
fn original_input_observation_retains_both_schemas_and_every_policy() {
    let original = inputs();
    let observed = RelationalOriginalInputObservation::from_recorded(&original);
    assert_eq!(observed.prestate_schema(), CanonicalDigestId::new([1; 32]));
    assert_eq!(observed.result_schema(), CanonicalDigestId::new([2; 32]));
    assert!(observed.introduced_schema().is_none());
    assert_eq!(
        observed.cascade_delete_policy(),
        original.mutation.cascade_delete_policy
    );
    assert_eq!(
        observed.adjacency_policy(),
        &original.mutation.adjacency_policy
    );
    assert_eq!(
        observed.cross_context_policy(),
        original.mutation.cross_context_policy
    );
    assert_eq!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&original)
    );
    let mut changed = inputs();
    changed
        .mutation
        .adjacency_policy
        .small_degree_inline_capacity += 1;
    assert_ne!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&changed)
    );
    let mut changed = inputs();
    changed.mutation.cascade_delete_policy = CascadeDeletePolicy::RetainDanglingForAudit;
    assert_ne!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&changed)
    );
    let mut changed = inputs();
    changed.mutation.cross_context_policy = CrossContextPolicy::Forbid;
    assert_ne!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&changed)
    );
    let mut changed = inputs();
    changed.prestate_schema[0] ^= 1;
    assert_ne!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&changed)
    );
    let mut changed = inputs();
    changed.result_schema[0] ^= 1;
    assert_ne!(
        observed,
        RelationalOriginalInputObservation::from_recorded(&changed)
    );
}
