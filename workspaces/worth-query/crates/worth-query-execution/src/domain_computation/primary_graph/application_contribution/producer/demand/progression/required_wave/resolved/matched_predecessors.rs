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
    /// Adds one matched edge to `matched`, starting the set on its first.
    pub(in crate::domain_computation::primary_graph) fn join(
        matched: &mut Option<Self>,
        registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
        consumer: &Arc<RecordedSettlementIdentity>,
        old_identity: Arc<RecordedSettlementIdentity>,
        successor: ReboundConsumedOutput,
        selected: &'a PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let item = std::mem::size_of::<MatchedConsumedRoot>();
        admission
            .charge_external_work(
                u64::try_from(item + std::mem::size_of::<Arc<RecordedSettlementIdentity>>() + 2)
                    .map_err(|_| work_denial())?,
            )
            .map_err(|_| work_denial())?;
        if matched.is_none() {
            *matched = Some(Self {
                _custody: registry.retain_consumer_handoff(consumer, admission)?,
                roots: Vec::new(),
                selected,
            });
        }
        let matched = matched.as_mut().expect("consumer wave custody established");
        if matched.roots.len() == matched.roots.capacity() {
            let next = matched.roots.capacity().saturating_mul(2).max(1);
            admission
                .admit_read_scratch(
                    u64::try_from(next.saturating_mul(item)).map_err(|_| capacity_denial())?,
                )
                .map_err(admission_denial)?;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.charge_external_work(
            (2 * std::mem::size_of::<RecordedSettlementIdentity>() + 1) as u64,
        )?;
        Ok(self._custody.matches(identity))
    }

    pub(in crate::domain_computation::primary_graph) fn root_count(&self) -> usize {
        self.roots.len()
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        admission.charge_external_work(4)?;
        let branch_work = self
            .selected
            .branch_id()
            .0
            .len()
            .checked_add(selected.branch_id().0.len())
            .and_then(|work| {
                work.checked_add(2 * std::mem::size_of::<PositionedRelationalSnapshot>())
            })
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(branch_work)?;
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
