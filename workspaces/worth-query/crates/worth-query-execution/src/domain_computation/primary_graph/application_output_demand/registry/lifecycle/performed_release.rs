//! A performed source nobody holds gives its observation back.
//!
//! A performed write retains its source, with the product observation it was
//! read at, so its required outputs start from that exact publication, now
//! or later by its receipt. A caller that drops the write before its outputs
//! start, or the started outputs before they advance, leaves that source
//! with no holder. A newer write of the same root replaces it; writes of
//! other roots do not, so abandoned sources could fill a branch for good.
//!
//! When the branch refuses a caller an observation and no closed cached row
//! is left to retire, the oldest such source is released. A row it admitted
//! keeps its obligation and is produced from the source its next demand
//! discloses. A recovery by the write's receipt then answers that the
//! publication retains no output custody, and an ordinary demand serves it.

use super::super::{DemandRegistryState, SourceCustody, WorthQueryPerformedOutputDemandSource};
use super::cached_reclaim::work_denial;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// What a released performed source kept. It drops after the registry lock
/// is released.
pub(super) struct ReleasedPerformedSource {
    _custody: SourceCustody,
    _admitted: Vec<WorthQueryPerformedOutputDemandSource>,
}

impl DemandRegistryState {
    /// Release the oldest retained performed source that no prepared token
    /// and no demand of a row it admitted holds.
    pub(super) fn release_unheld_performed_source(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ReleasedPerformedSource>, WorthQueryOutputDemandDenial> {
        let mut charge = |work: usize| {
            u64::try_from(work)
                .ok()
                .and_then(|work| admission.charge_external_work(work).ok())
                .ok_or_else(work_denial)
        };
        // One pass over the sources; each candidate compares the rows once.
        charge(self.source_custody.len())?;
        let mut oldest = None;
        for (commit, custody) in &self.source_custody {
            if custody.token_count != 0
                || custody.source.is_none()
                || oldest.is_some_and(|oldest| oldest < commit)
            {
                continue;
            }
            charge(self.records.len())?;
            if !self.records.values().any(|record| {
                record.source_commits.contains(commit)
                    && (record.interests != 0 || record.required_interests != 0)
            }) {
                oldest = Some(commit);
            }
        }
        let Some(commit) = oldest.cloned() else {
            return Ok(None);
        };
        charge(self.records.len())?;
        let custody = self
            .source_custody
            .remove(&commit)
            .expect("the selected source is retained");
        let mut admitted = Vec::new();
        for record in self.records.values_mut() {
            if record
                .performed_source
                .as_ref()
                .is_some_and(|source| source.change.product_commit() == &commit)
            {
                admitted.extend(record.performed_source.take());
            }
            record.source_commits.retain(|retained| retained != &commit);
        }
        Ok(Some(ReleasedPerformedSource {
            _custody: custody,
            _admitted: admitted,
        }))
    }
}
