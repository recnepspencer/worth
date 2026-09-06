use super::*;

impl ProductUnpublishedOwnerEffectsRecord {
    pub(crate) fn from_abandoned(
        facts: RetainedAttemptFacts,
        successor_basis: Option<AdmittedCompositeRuntimeWorldBasis>,
        resources: crate::publication::ActiveAttemptResources,
        catalog_affinity: usize,
        deadline: Option<RuntimeWorldInstant>,
        cause: ProductUnpublishedCause,
    ) -> Arc<Self> {
        let next_actions =
            RetainedNextActions::from_vec(next_actions_for_progress(&facts.progress, cause));
        let owner_effect_count = facts.progress.owner_effect_count();
        let live_obligations = resources.live_obligations();
        Arc::new(Self {
            identity: facts.identity,
            attempt_identity: facts.attempt_identity,
            expected_head: facts.expected_head,
            last_observed_head: facts.last_observed_head,
            progress: facts.progress,
            successor_basis,
            component_results: facts.owner_results,
            retention: resources,
            destination: facts.destination,
            catalog_affinity,
            live_obligations,
            cause,
            next_actions,
            deadline,
            admitted_at: facts.admitted_at,
            owner_effect_count,
            metadata_bytes: ProductUnpublishedOwnerEffects::metadata_charge_hint(),
        })
    }
}
