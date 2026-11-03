use std::sync::Arc;

use super::DemandRegistryState;
use super::WorthQueryOutputDemandKey;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

#[derive(Default)]
pub(super) struct DetachedRequiredMember {
    pub(super) _source: Option<Arc<super::source_readmission::RequiredOutputReadmission>>,
    pub(super) _key: Option<Arc<WorthQueryOutputDemandKey>>,
}

pub(super) struct PreparedRequiredMember {
    key: Arc<WorthQueryOutputDemandKey>,
}

impl DemandRegistryState {
    /// A queued member is derived custody, not proof that its count is live.
    /// Readers without a drain owner check the count and stale owners using
    /// their own admission before treating that queued key as required.
    pub(super) fn queued_member_released(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let Some(_) = self
            .records
            .get(key)
            .filter(|row| row.pending_cleanup_queued || row.closed_cleanup.queued())
        else {
            return Ok(false);
        };
        let observation = super::claim_readthrough::observe_row(&self.records, key);
        Ok(!observation.required && !observation.awaited)
    }

    pub(super) fn prepare_required_member(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<PreparedRequiredMember>, WorthQueryOutputDemandDenial> {
        if self.required_keys.contains(key) {
            return Ok(None);
        }
        Ok(Some(PreparedRequiredMember {
            key: Arc::new(key.clone()),
        }))
    }

    pub(super) fn install_required_member(&mut self, prepared: Option<PreparedRequiredMember>) {
        if let Some(prepared) = prepared {
            debug_assert!(self.records.contains_key(prepared.key.as_ref()));
            let membership = self
                .records
                .get(prepared.key.as_ref())
                .and_then(|record| record.work_membership.as_ref())
                .cloned();
            self.required_keys.insert(prepared.key);
            if let Some(membership) = membership {
                membership.set_required(true);
            }
        }
    }

    pub(super) fn remove_required_member_if_released(&mut self, key: &WorthQueryOutputDemandKey) {
        let retired = self.remove_required_member_if_released_detached(key);
        drop(retired);
    }

    /// Selected terminal cleanup carries the real readmission source out of
    /// the registry guard before its final Arc can be destroyed.
    pub(super) fn remove_required_member_if_released_detached(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> DetachedRequiredMember {
        if self
            .records
            .get(key)
            .is_some_and(|record| record.is_required())
        {
            return DetachedRequiredMember::default();
        }
        let awaited = super::refreshed_rejoin::awaited_by_stale_owner(&self.records, key);
        self.remove_required_member_with_awaited(key, awaited)
    }

    pub(super) fn remove_required_member_with_awaited(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        awaited: bool,
    ) -> DetachedRequiredMember {
        if self
            .records
            .get(key)
            .is_some_and(|record| record.is_required() || awaited)
        {
            return DetachedRequiredMember::default();
        }
        self.note_record_custody(key);
        let mut retired_source = None;
        if let Some(mut record) = self.records.get_mut(key) {
            // A reusable cached Ready keeps its original admitted source, funded
            // by that source's existing ticket, until the cache row is reclaimed.
            let reusable = matches!(&record.state, super::DemandState::Output(output)
                if matches!(output.advancement(), super::WorthQueryOutputAdvancement::Idle)
                    && record.has_cached_ready());
            if !reusable {
                retired_source = record.readmission_source.take();
            }
            if let Some(membership) = &record.work_membership {
                membership.set_required(false);
            }
        }
        let removed_key = self.required_keys.take(key);
        DetachedRequiredMember {
            _source: retired_source,
            _key: removed_key,
        }
    }
}
