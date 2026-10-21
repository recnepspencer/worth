//! Required-only replay of a genuine native discontinuity, one key per step.
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_runtime_world::facade::ProductBranchIncarnation;
#[derive(Default)]
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct DiscontinuityCursors
{
    occurrences: BTreeMap<ProductBranchIncarnation, BranchCursors>,
}
pub(super) enum DiscontinuityPage {
    Queued,
    Advanced,
    Complete,
}
struct BranchCursors {
    branches: BTreeMap<BranchId, DiscontinuityCursor>,
}
struct DiscontinuityCursor {
    epoch: CommitId,
    last_key: Option<Arc<WorthQueryOutputDemandKey>>,
    complete: bool,
}
/// Detached under the registry lock; its actual branch/key backing drops after it.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct RetiredDiscontinuityCursors
{
    _row: BranchCursors,
}
impl DemandRegistryState {
    pub(super) fn page_required_discontinuity(
        &mut self,
        occurrence: ProductBranchIncarnation,
        branch: &BranchId,
        epoch: CommitId,
    ) -> Result<DiscontinuityPage, WorthQueryOutputDemandDenial> {
        let previous = self
            .discontinuity_cursors
            .occurrences
            .get(&occurrence)
            .and_then(|row| row.branches.get(branch));
        let continuing = previous.is_some_and(|cursor| cursor.epoch == epoch);
        if previous.is_some_and(|cursor| continuing && cursor.complete) {
            return Ok(DiscontinuityPage::Complete);
        }
        let last = previous
            .filter(|_| continuing)
            .and_then(|cursor| cursor.last_key.as_ref());
        let selected_key = match last {
            Some(last) => self
                .required_keys
                .range((Excluded(Arc::clone(last)), Unbounded))
                .next(),
            None => self.required_keys.iter().next(),
        }
        .cloned();
        let selected_member = if let Some(key) = selected_key.as_ref() {
            let record = self.records.get(key.as_ref());
            let still_required = !self.queued_member_released(key)?;
            record.and_then(|record| {
                (still_required && record.product_occurrence == occurrence)
                    .then(|| record.work_membership.as_ref().cloned())
                    .flatten()
            })
        } else {
            None
        };
        let row = self
            .discontinuity_cursors
            .occurrences
            .entry(occurrence)
            .or_insert_with(|| BranchCursors {
                branches: BTreeMap::new(),
            });
        let cursor = row
            .branches
            .entry(branch.clone())
            .or_insert(DiscontinuityCursor {
                epoch,
                last_key: None,
                complete: false,
            });
        cursor.epoch = epoch;
        cursor.last_key = selected_key;
        cursor.complete = cursor.last_key.is_none();
        if let Some(member) = selected_member {
            member.mark_discontinuity();
            return Ok(DiscontinuityPage::Queued);
        }
        Ok(if cursor.complete {
            DiscontinuityPage::Complete
        } else {
            DiscontinuityPage::Advanced
        })
    }
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn take_discontinuity_cursors(
        &mut self,
        occurrence: ProductBranchIncarnation,
    ) -> Option<RetiredDiscontinuityCursors> {
        self.discontinuity_cursors
            .occurrences
            .remove(&occurrence)
            .map(|row| RetiredDiscontinuityCursors { _row: row })
    }
}
