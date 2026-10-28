//! Unit-only epoch construction; production epochs come from admitted observations.
use super::*;

impl WorthQueryObservedSourceEpoch {
    pub(in crate::domain_computation::primary_graph) fn new(
        query: [u8; 32],
        parameters: [u8; 32],
        root: worth_relational::facade::identity::EntityId,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        observation_generation: u64,
        identity: [u8; 32],
    ) -> Self {
        let footprint = WorthQueryObservedSourceFootprint {
            root,
            complete: true,
            entities: vec![root],
            aspects: Vec::new(),
            adjacencies: Vec::new(),
            root_selection: None,
        };
        let checkpoint_identity =
            checkpoint_commitment::checkpoint_source_identity(&query, &parameters, &footprint);
        Self {
            query,
            parameters,
            root,
            occurrence,
            observation_generation,
            meaning: Arc::new(WorthQueryObservedSourceMeaning {
                coordinate: WorthQueryObservedSourceCoordinate {
                    query,
                    parameters,
                    occurrence: WorthQueryObservedSourceOccurrence::Product(occurrence),
                    root,
                },
                footprint,
                identity,
                runtime_idempotency_identity: identity,
                checkpoint_identity,
                registry: Weak::new(),
            }),
        }
    }
}
