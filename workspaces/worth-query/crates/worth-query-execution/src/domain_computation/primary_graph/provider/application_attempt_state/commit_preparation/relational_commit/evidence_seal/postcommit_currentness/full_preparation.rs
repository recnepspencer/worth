use super::*;
/// Capacity for the selected-snapshot fact transition is acquired while the
/// candidate is still pre-effect custody. The postcommit path only fills the
/// already allocated action and result buffers.
pub(super) struct FullSourceFactRebase {
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
    /// The ordinals, ascending, of the facts an owner call read that the
    /// commit's own effect replaced.
    pub(super) moved_by_own_effect: Arc<[usize]>,
    pub(super) endpoints: worth_execution::ExecutionArray<(
        usize,
        crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints,
    )>,
    pub(super) actions: Vec<resolution::PreparedFactRebase>,
    pub(super) rebased: NativeFactPayloadBuilder,
    pub(super) superseded: Vec<usize>,
}

impl FullSourceFactRebase {
    pub(in crate::domain_computation::primary_graph) fn admit(
        facts: Vec<WorthQueryApplicationObservedFact>,
        moved_by_own_effect: Arc<[usize]>,
        control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        let endpoint_count = facts.iter().try_fold(0usize, |count, fact| {
            fact_verification_observation::rebase_preparation();
            control.check_live()?;
            if matches!(fact, WorthQueryApplicationObservedFact::Relation { .. } | WorthQueryApplicationObservedFact::Adjacency { .. }) {
                count.checked_add(1).ok_or(crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Representability)
            } else { Ok(count) }
        })?;
        let mut endpoints =
            worth_execution::ExecutionArrayBuilder::allocate(endpoint_count, control.policy())?;
        for (ordinal, fact) in facts.iter().enumerate() {
            fact_verification_observation::rebase_preparation();
            control.check_live()?;
            if let Some(payload) = adjacency::prepare_endpoints(fact, control)? {
                endpoints.push((ordinal, payload))?;
            }
        }
        let endpoints = endpoints.seal()?;
        let mut actions = Vec::new();
        actions.try_reserve_exact(facts.len())?;
        let rebased = NativeFactPayloadBuilder::allocate(facts.len(), control)?;
        let mut superseded = Vec::new();
        superseded.try_reserve_exact(facts.len())?;
        Ok(Self {
            facts,
            moved_by_own_effect,
            endpoints,
            actions,
            rebased,
            superseded,
        })
    }
}
