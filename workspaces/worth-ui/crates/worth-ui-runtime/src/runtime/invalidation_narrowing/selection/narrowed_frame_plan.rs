use crate::runtime::{
    UiAllocationFramePlanIdentity, UiAllocationInvalidationFamily, UiAllocationInvalidationIntent,
    UiAllocationInvalidationReferenceDenial,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiAllocationInvalidationNarrowingCounters {
    invalidation_visits: u16,
    query_settlement_visits: u16,
    query_observation_visits: u16,
    graph_target_lookups: u16,
    authority_probes: u16,
    emitted_targets: u16,
    materialized_host_target_sets: u16,
}

#[derive(Debug, PartialEq)]
pub struct UiNarrowedAllocationFramePlan {
    frame_identity: UiAllocationFramePlanIdentity,
    resolution_counters: crate::runtime::UiAllocationFrameResolutionCounters,
    invalidations: Box<[super::UiNarrowedAllocationInvalidation]>,
    counters: UiAllocationInvalidationNarrowingCounters,
}

impl UiNarrowedAllocationFramePlan {
    pub(super) fn new(
        frame_identity: UiAllocationFramePlanIdentity,
        resolution_counters: crate::runtime::UiAllocationFrameResolutionCounters,
        invalidations: Box<[super::UiNarrowedAllocationInvalidation]>,
        counters: UiAllocationInvalidationNarrowingCounters,
    ) -> Self {
        Self {
            frame_identity,
            resolution_counters,
            invalidations,
            counters,
        }
    }
    pub fn frame_identity(&self) -> &UiAllocationFramePlanIdentity {
        &self.frame_identity
    }
    pub fn identity(&self) -> &UiAllocationFramePlanIdentity {
        &self.frame_identity
    }
    pub fn invalidations(&self) -> &[UiAllocationInvalidationIntent] {
        self.frame_identity.invalidations()
    }
    pub fn narrowed_invalidations(&self) -> &[super::UiNarrowedAllocationInvalidation] {
        &self.invalidations
    }
    pub(crate) fn resize_preview_sample(&self) -> Option<crate::runtime::UiResizePreviewSample> {
        self.invalidations
            .iter()
            .rev()
            .find_map(|invalidation| match invalidation.target() {
                super::UiAllocationInvalidationTarget::ResizePreview { sample, .. } => {
                    Some(*sample)
                }
                super::UiAllocationInvalidationTarget::Graph(_)
                | super::UiAllocationInvalidationTarget::SettledQueryFact { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedContentExtent { .. }
                | super::UiAllocationInvalidationTarget::HostMeasurement { .. }
                | super::UiAllocationInvalidationTarget::PortalAnchor { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedExtent { .. }
                | super::UiAllocationInvalidationTarget::DurableResize { .. } => None,
            })
    }
    pub(crate) fn resize_preview_sample_count(&self) -> u16 {
        u16::try_from(
            self.frame_identity
                .invalidations()
                .iter()
                .filter(|invalidation| {
                    invalidation.family() == UiAllocationInvalidationFamily::ResizePreviewDelta
                })
                .count(),
        )
        .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress")
    }
    pub(crate) fn durable_resize_extent(&self) -> Option<crate::runtime::UiResizeLogicalExtent> {
        self.invalidations
            .iter()
            .find_map(|invalidation| match invalidation.target() {
                super::UiAllocationInvalidationTarget::DurableResize { extent, .. } => {
                    Some(*extent)
                }
                super::UiAllocationInvalidationTarget::Graph(_)
                | super::UiAllocationInvalidationTarget::ResizePreview { .. }
                | super::UiAllocationInvalidationTarget::SettledQueryFact { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedContentExtent { .. }
                | super::UiAllocationInvalidationTarget::HostMeasurement { .. }
                | super::UiAllocationInvalidationTarget::PortalAnchor { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedExtent { .. } => None,
            })
    }
    pub(crate) fn durable_resize_identity_digest(&self) -> Option<u64> {
        self.invalidations
            .iter()
            .find_map(|invalidation| match invalidation.target() {
                super::UiAllocationInvalidationTarget::DurableResize {
                    identity_digest, ..
                } => Some(*identity_digest),
                super::UiAllocationInvalidationTarget::Graph(_)
                | super::UiAllocationInvalidationTarget::ResizePreview { .. }
                | super::UiAllocationInvalidationTarget::SettledQueryFact { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedContentExtent { .. }
                | super::UiAllocationInvalidationTarget::HostMeasurement { .. }
                | super::UiAllocationInvalidationTarget::PortalAnchor { .. }
                | super::UiAllocationInvalidationTarget::ScrollOwnedExtent { .. } => None,
            })
    }
    pub fn counters(&self) -> UiAllocationInvalidationNarrowingCounters {
        self.counters
    }
    pub fn resolution_counters(&self) -> crate::runtime::UiAllocationFrameResolutionCounters {
        self.resolution_counters
    }
    pub fn families(&self) -> &[crate::runtime::UiAllocationStreamFamily] {
        self.frame_identity.families()
    }
    pub fn narrowed_families(
        &self,
    ) -> impl ExactSizeIterator<Item = UiAllocationInvalidationFamily> + '_ {
        self.invalidations.iter().map(|value| value.family())
    }
    pub fn frame_epoch(&self) -> crate::runtime::UiAllocationFrameEpoch {
        self.frame_identity.epoch()
    }
    pub fn policy(&self) -> crate::runtime::UiResolvedAllocationStreamPolicy {
        self.frame_identity.policy()
    }
    pub fn cadence_verdict(&self) -> crate::runtime::UiAllocationFrameCadenceVerdict {
        self.frame_identity.cadence()
    }
    pub fn evidence(&self) -> &crate::evidence::UiAllocationStreamPolicyEvidenceOutcome {
        self.frame_identity.evidence()
    }
    pub fn order_verdicts(&self) -> &[crate::runtime::UiAllocationSourceOrderVerdict] {
        self.frame_identity.order_verdicts()
    }
    pub fn ingress_policy_verdicts(&self) -> &[crate::runtime::UiAllocationIngressPolicyVerdict] {
        self.frame_identity.ingress_policy_verdicts()
    }
    pub fn duplicate_posture(&self) -> crate::runtime::UiAllocationDuplicatePosture<'_> {
        self.frame_identity.duplicate_posture()
    }
    pub fn invalidation_ingress_key(
        &self,
        invalidation: &UiAllocationInvalidationIntent,
    ) -> Result<&crate::runtime::UiAllocationFrameIngressKey, UiAllocationInvalidationReferenceDenial>
    {
        let reference = invalidation.ingress_ref();
        if reference.epoch() != self.frame_epoch() {
            return Err(
                UiAllocationInvalidationReferenceDenial::FrameEpochMismatch {
                    plan: self.frame_epoch(),
                    invalidation: reference.epoch(),
                },
            );
        }
        let ordinal = usize::from(reference.ordinal());
        let owned = self.invalidations().get(ordinal).ok_or(
            UiAllocationInvalidationReferenceDenial::MissingCanonicalIngress {
                ordinal: reference.ordinal(),
                ingress_count: u16::try_from(self.invalidations().len())
                    .expect("a frame carries at most ALLOCATION_FRAME_SOURCE_COUNT ingress"),
            },
        )?;
        if !std::ptr::eq(owned, invalidation) {
            return Err(
                UiAllocationInvalidationReferenceDenial::ForeignPlanInvalidation {
                    plan: self.frame_epoch(),
                    invalidation: reference.epoch(),
                    ordinal: reference.ordinal(),
                },
            );
        }
        Ok(&self.frame_identity.ingress_keys()[ordinal])
    }
}

impl UiAllocationInvalidationNarrowingCounters {
    pub(super) fn visit_invalidation(&mut self) -> Result<(), ()> {
        increment(&mut self.invalidation_visits, 1)
    }
    pub(super) fn visit_query_settlement(&mut self) -> Result<(), ()> {
        increment(&mut self.query_settlement_visits, 1)
    }
    pub(super) fn visit_query_observations(&mut self, count: usize) -> Result<(), ()> {
        increment(&mut self.query_observation_visits, count)
    }
    pub(super) fn lookup_graph_target(&mut self) -> Result<(), ()> {
        increment(&mut self.graph_target_lookups, 1)
    }
    pub(super) fn record_authority_probes(&mut self, count: u16) -> Result<(), ()> {
        increment(&mut self.authority_probes, usize::from(count))
    }
    pub(super) fn emit_targets(&mut self, count: usize) -> Result<(), ()> {
        increment(&mut self.emitted_targets, count)
    }
    pub(super) fn materialize_host_target_set(&mut self) -> Result<(), ()> {
        increment(&mut self.materialized_host_target_sets, 1)
    }
    pub fn invalidation_visits(self) -> u16 {
        self.invalidation_visits
    }
    pub fn query_settlement_visits(self) -> u16 {
        self.query_settlement_visits
    }
    pub fn query_observation_visits(self) -> u16 {
        self.query_observation_visits
    }
    pub fn graph_target_lookups(self) -> u16 {
        self.graph_target_lookups
    }
    pub fn authority_probes(self) -> u16 {
        self.authority_probes
    }
    pub fn emitted_targets(self) -> u16 {
        self.emitted_targets
    }
    pub fn materialized_host_target_sets(self) -> u16 {
        self.materialized_host_target_sets
    }
}

fn increment(target: &mut u16, count: usize) -> Result<(), ()> {
    let count = u16::try_from(count).map_err(|_| ())?;
    *target = target.checked_add(count).ok_or(())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::UiAllocationInvalidationNarrowingCounters;

    #[test]
    fn emitted_target_counter_records_exact_handoff_width() {
        let mut counters = UiAllocationInvalidationNarrowingCounters::default();
        counters.emit_targets(3).unwrap();
        counters.emit_targets(2).unwrap();
        assert_eq!(counters.emitted_targets(), 5);
    }
}
