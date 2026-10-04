use std::sync::Arc;

use super::{DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

#[derive(Default)]
pub(super) struct DetachedRequiredMember {
    pub(super) _source: Option<Arc<super::source_readmission::RequiredOutputReadmission>>,
    pub(super) _key: Option<Arc<WorthQueryOutputDemandKey>>,
    pub(super) refund_required_bytes: usize,
}

/// One ordered-tree node per member covers splits without charging unrelated
/// registry records. The Arc owns a second key; SourceEpoch's clone shares its
/// already admitted meaning, while producer text is copied into the new key.
pub(super) fn member_bytes(key: &WorthQueryOutputDemandKey) -> Option<usize> {
    minimum_member_bytes()?.checked_add(key.producer.len())
}

pub(super) fn minimum_member_bytes() -> Option<usize> {
    let tree_node = 12usize
        .checked_mul(std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>())?
        .checked_add(14usize.checked_mul(std::mem::size_of::<usize>())?)?;
    std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>()
        .checked_add(std::mem::size_of::<WorthQueryOutputDemandKey>())?
        .checked_add(2 * std::mem::size_of::<usize>())?
        .checked_add(tree_node)
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required output index has no retained capacity",
    )
}

pub(super) struct PreparedRequiredMember {
    key: Arc<WorthQueryOutputDemandKey>,
    bytes: usize,
}

impl PreparedRequiredMember {
    pub(super) const fn reserved_bytes(&self) -> usize {
        self.bytes
    }
}

impl DemandRegistryState {
    pub(super) fn prepare_required_member(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Result<Option<PreparedRequiredMember>, WorthQueryOutputDemandDenial> {
        if self.required_keys.contains(key) {
            return Ok(None);
        }
        let bytes = member_bytes(key).ok_or_else(capacity_denial)?;
        if self
            .required_reserved_bytes
            .checked_add(bytes)
            .is_none_or(|required| !self.has_required_capacity(required))
        {
            return Err(super::required_custody::full_custody_denial(
                bytes,
                self.required_budget_bytes,
            ));
        }
        Ok(Some(PreparedRequiredMember {
            key: Arc::new(key.clone()),
            bytes,
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
            if self.required_keys.insert(prepared.key) {
                self.required_reserved_bytes += prepared.bytes;
            }
            if let Some(membership) = membership {
                membership.set_required(true);
            }
        }
    }

    pub(super) fn remove_required_member_if_released(&mut self, key: &WorthQueryOutputDemandKey) {
        let retired = self.remove_required_member_if_released_detached(key);
        let refund = retired.refund_required_bytes;
        drop(retired);
        self.required_reserved_bytes = self.required_reserved_bytes.saturating_sub(refund);
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
            .is_some_and(super::DemandRecord::is_required)
            || super::refreshed_rejoin::awaited_by_stale_owner(&self.records, key)
        {
            return DetachedRequiredMember::default();
        }
        let mut retired_source = None;
        if let Some(record) = self.records.get_mut(key) {
            retired_source = record.readmission_source.take();
            if let Some(membership) = &record.work_membership {
                membership.set_required(false);
            }
        }
        let removed_key = self.required_keys.take(key);
        let refund_required_bytes = removed_key
            .as_ref()
            .map(|member| member_bytes(member.as_ref()).expect("admitted key charge fits"))
            .unwrap_or(0);
        DetachedRequiredMember {
            _source: retired_source,
            _key: removed_key,
            refund_required_bytes,
        }
    }
}
