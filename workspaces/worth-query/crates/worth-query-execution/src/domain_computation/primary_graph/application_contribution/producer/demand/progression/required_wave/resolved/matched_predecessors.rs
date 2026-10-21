//! Exact consumer custody and the successors matched to its consumed roots.
use super::*;

/// Old consumed identities, each joined to the successor row whose exact
/// Ready was already certified Current on this selected wave, whether this
/// wave or an earlier advance refreshed it. A consumer of several outputs
/// carries one per resolved consumed edge.
pub(in crate::domain_computation::primary_graph) struct MatchedRequiredPredecessors<'a> {
    _custody: crate::domain_computation::primary_graph::application_output_demand::ConsumerCustody,
    roots: Vec<MatchedConsumedRoot>,
    selected: &'a PositionedRelationalSnapshot,
}

pub(in crate::domain_computation::primary_graph) enum ReboundConsumedOutput {
    Equal(ConsumedOutputEvidence),
    Fresh,
}
struct MatchedConsumedRoot {
    old: Arc<RecordedSettlementIdentity>,
    rebound: ReboundConsumedOutput,
}

impl<'a> MatchedRequiredPredecessors<'a> {
    /// Preserve the exact posted consumer before a source-only refresh retires
    /// it. No upstream has been matched: all original roots still require the
    /// ordinary native verification at this pinned selected snapshot.
    pub(in crate::domain_computation::primary_graph) fn retain_consumer(
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
        consumer: &Arc<RecordedSettlementIdentity>,
        selected: &'a PositionedRelationalSnapshot,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        Ok(Self {
            _custody: registry.retain_consumer_handoff(consumer)?,
            roots: Vec::new(),
            selected,
        })
    }
    /// Adds one matched edge to `matched`, starting the set on its first.
    pub(in crate::domain_computation::primary_graph) fn join(
        matched: &mut Option<Self>,
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
        consumer: &Arc<RecordedSettlementIdentity>,
        old_identity: Arc<RecordedSettlementIdentity>,
        successor: ReboundConsumedOutput,
        selected: &'a PositionedRelationalSnapshot,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if matched.is_none() {
            *matched = Some(Self::retain_consumer(registry, consumer, selected)?);
        }
        let matched = matched.as_mut().expect("consumer wave custody established");
        if matched.roots.len() == matched.roots.capacity() {
            let next = matched.roots.capacity().saturating_mul(2).max(1);

            matched
                .roots
                .try_reserve_exact(next - matched.roots.len())
                .map_err(|_| capacity_denial())?;
        }

        matched.roots.push(MatchedConsumedRoot {
            old: old_identity,
            rebound: successor,
        });
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn matches_consumer(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<bool, CompanionPreflightStop> {
        Ok(self._custody.matches(identity))
    }

    pub(in crate::domain_computation::primary_graph) fn old_identities(
        &self,
    ) -> impl Iterator<Item = &Arc<RecordedSettlementIdentity>> {
        self.roots.iter().map(|root| &root.old)
    }
    pub(in crate::domain_computation::primary_graph) fn replacement(
        &self,
        old: &Arc<RecordedSettlementIdentity>,
    ) -> Option<&ConsumedOutputEvidence> {
        self.roots
            .iter()
            .find(|root| &root.old == old)
            .and_then(|root| match &root.rebound {
                ReboundConsumedOutput::Equal(evidence) => Some(evidence),
                ReboundConsumedOutput::Fresh => None,
            })
    }

    /// The cutoff constructs its own positioned view of the issued snapshot.
    /// Join its coordinates to this invocation's certified wave before using
    /// the old consumed identities to choose a fresh handler path.
    pub(in crate::domain_computation::primary_graph) fn matches_cutoff_root(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        Ok(
            self.selected.runtime_instance_id() == selected.runtime_instance_id()
                && self.selected.branch_id() == selected.branch_id()
                && self.selected.root_id() == selected.root_id()
                && self.selected.version_id() == selected.version_id()
                && self.selected.commit_id() == selected.commit_id()
                && self.selected.position() == selected.position(),
        )
    }
}
