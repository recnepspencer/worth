use std::sync::Arc;

use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

use super::DemandRecord;
use super::DemandRegistryState;
use super::DemandState;
use super::WorthQueryOutputCheckpoint;
use super::WorthQueryOutputDemandKey;

pub(super) type ReleasedPrerequisiteResources = (
    Vec<Arc<WorthQueryOutputDemandKey>>,
    Vec<Arc<RecordedSettlementIdentity>>,
    Option<super::prerequisite_claims::CheckpointPrerequisiteClaims>,
);

impl DemandRecord {
    pub(super) fn release_obligations(&mut self) {
        self.performed_obligations = Vec::new();
    }

    pub(super) fn is_required(&self) -> bool {
        self.interests != 0
            || self.required_interests != 0
            || !self.performed_obligations.is_empty()
            || self.framework_required_count != 0
    }

    pub(super) fn has_cached_ready(&self) -> bool {
        matches!(&self.state, DemandState::Output(output)
            if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(_))))
    }
}

impl DemandRegistryState {
    /// Release one terminal output's framework claims while the registry lock
    /// is held. Return the old Arc pointers for destruction after that lock.
    pub(super) fn release_record_prerequisites(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Vec<Arc<WorthQueryOutputDemandKey>> {
        let (prior, settlements, checkpoint) = self.release_record_prerequisites_inner(key, None);
        drop(settlements);
        // Static claim final destruction was prepaid at its admission. It
        // contains only settlement addresses and an atomic refund ticket.
        drop(checkpoint);

        prior
    }

    /// The rows a released dependent claim on `upstream` may have been the
    /// last to require: the row itself, and the newest row of its occurrence
    /// when a refresh superseded it.
    fn released_by_claim(
        &self,
        upstream: &WorthQueryOutputDemandKey,
    ) -> impl Iterator<Item = WorthQueryOutputDemandKey> + use<> {
        let newest = super::refreshed_rejoin::newest_of_occurrence(&self.records, upstream);
        std::iter::once(upstream.clone()).chain(newest)
    }

    fn release_claim_members(
        &mut self,
        upstream: &Arc<WorthQueryOutputDemandKey>,
        mut retired: Option<&mut Vec<super::required_members::DetachedRequiredMember>>,
    ) {
        for released in self.released_by_claim(upstream) {
            // The awaiting fact is read once after the claim decrement and
            // carried through both membership and cleanup decisions.
            let awaited = super::refreshed_rejoin::awaited_by_stale_owner(&self.records, &released);
            let funded_key = self.required_keys.get(&released).cloned();
            let abandoned = self.records.get(&released).is_some_and(|record| {
                matches!(
                    record.idle_disposition(awaited),
                    super::IdleRowDisposition::Retired(reason) if reason.releases_finished_work() && record.interests == 0
                ) || record.successor_is_orphaned(awaited)
            });
            if abandoned
                && funded_key
                    .as_ref()
                    .is_some_and(|key| self.defer_released_member_cleanup(key, awaited))
            {
                continue;
            }
            let member = self.remove_required_member_with_awaited(&released, awaited);
            if let Some(retired) = retired.as_deref_mut() {
                retired.push(member);
            } else {
                drop(member);
            }
            if &released == upstream.as_ref() {
                self.defer_terminal_cleanup_with_awaited(upstream, awaited);
            }
        }
    }

    pub(super) fn release_record_prerequisites_detached(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        retired_members: &mut Vec<super::required_members::DetachedRequiredMember>,
    ) -> ReleasedPrerequisiteResources {
        self.release_record_prerequisites_inner(key, Some(retired_members))
    }

    fn release_record_prerequisites_inner(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        mut retired_members: Option<&mut Vec<super::required_members::DetachedRequiredMember>>,
    ) -> ReleasedPrerequisiteResources {
        let Some(mut record) = self.records.get_mut(key) else {
            return (Vec::new(), Vec::new(), None);
        };
        if record.prepared_prerequisite_claims != 0
            || record.framework_required_count != 0
            || record.pending_cleanup_queued
        {
            return (Vec::new(), Vec::new(), None);
        }
        record.claim_batch.take();
        let prior = std::mem::take(&mut record.prerequisites);
        let checkpoint = record.checkpoint_prerequisites.take();
        let settlements = std::mem::take(&mut record.settlements);
        crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
            crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::PostingRelease,
            settlements.len(), prior.len(),
        );
        record.finish();
        for identity in &settlements {
            self.settlement_keys.remove(identity);
        }
        for upstream in &prior {
            let mut record = self
                .records
                .get_mut(upstream.as_ref())
                .expect("framework prerequisite record remains retained");
            record.framework_required_count -= 1;
            record.finish();
            self.release_claim_members(upstream, retired_members.as_deref_mut());
        }
        for upstream in &prior {
            self.defer_terminal_cleanup(upstream);
        }
        (prior, settlements, checkpoint)
    }

    /// A branch-wide retirement already visits its records. Move the retired
    /// prerequisite vectors out in that walk, then release their upstream
    /// claims in a second pass so no record is mutably aliased with another.
    pub(super) fn release_matching_prerequisites(
        &mut self,
        retired: impl Fn(&WorthQueryOutputDemandKey, &DemandRecord) -> bool,
    ) {
        let count = self
            .records
            .iter()
            .filter(|(key, record)| {
                #[cfg(test)]
                super::post_commit::record_sweep_contact();
                retired(key, record)
                    && record.prepared_prerequisite_claims == 0
                    && record.framework_required_count == 0
                    && !record.pending_cleanup_queued
                    && (!record.prerequisites.is_empty() || record.settlements.capacity() != 0)
            })
            .count();
        let mut released = Vec::with_capacity(count);
        let mut settlements = Vec::with_capacity(count);
        for (key, mut record) in &mut self.records {
            #[cfg(test)]
            super::post_commit::record_sweep_contact();
            if retired(key, &record)
                && record.prepared_prerequisite_claims == 0
                && record.framework_required_count == 0
                && !record.pending_cleanup_queued
            {
                record.checkpoint_prerequisites.take();
                if !record.prerequisites.is_empty() {
                    record.claim_batch.take();
                    released.push(std::mem::take(&mut record.prerequisites));
                }
                if record.settlements.capacity() != 0 {
                    settlements.push(std::mem::take(&mut record.settlements));
                }
            }
        }
        for settled in &settlements {
            crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
                crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::PostingRelease,
                settled.len(), 0,
            );

            for identity in settled {
                self.settlement_keys.remove(identity);
            }
        }
        for prior in &released {
            for upstream in prior {
                let mut record = self
                    .records
                    .get_mut(upstream.as_ref())
                    .expect("framework prerequisite record remains retained");
                record.framework_required_count -= 1;
                record.finish();
                self.release_claim_members(upstream, None);
            }
        }
        for prior in &released {
            for upstream in prior {
                self.defer_terminal_cleanup(upstream);
            }
        }
    }
}

impl super::WorthQueryAcceptedOutputAuthority {
    pub(super) fn is_same_settlement_authority(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Committed(left), Self::Committed(right)) => {
                left.is_same_authoritative_commit(right)
            }
            (Self::Stable(left), Self::Stable(right)) => left.same_publication(right),
            (Self::Restored(left), Self::Restored(right)) => {
                left.checkpoint == right.checkpoint
                    && std::sync::Arc::ptr_eq(&left.correspondence, &right.correspondence)
                    && left.observation == right.observation
                    && left.source_scope == right.source_scope
                    && left.source_identity == right.source_identity
                    && left
                        .observed_source_facts
                        .same_backing(&right.observed_source_facts)
            }
            _ => false,
        }
    }
}
