//! Query-read custody in the SAME native physical posting index, never a producer.
use super::fact_key::FactPostingKey;
use super::settlement::PreparedPostingOrdinals;
use super::source_alignment::SnapshotAlignedMarkState;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
use im::OrdMap;
use im::OrdSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
mod marking;
mod reclamation;
mod registration;
use reclamation::QueryReadBacking;
#[cfg(test)]
mod tests;
pub(super) use marking::mark_query_reads;

/// Addresses and their allocation only. No source/session/currentness permission.
#[derive(Debug)]
pub(super) struct QueryReadProjection {
    keys: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
}
#[derive(Debug)]
struct QueryReadIdentity;

/// Runtime-only custody minted from an original native query capture. Clones
/// share the one registration; the last owner removes its live index binding.
/// The slot is a locator only: native identity, epoch and selected image decide.
pub(in crate::domain_computation::primary_graph) struct QueryFactCoverage {
    owner: Arc<SourceInvalidationOwner>,
    basis: PositionedRelationalSnapshot,
    projection: Arc<QueryReadProjection>,
    identity: Arc<QueryReadIdentity>,
    slot: Mutex<Option<u64>>,
}
#[derive(Clone, Debug)]
pub(super) struct QueryReadBinding {
    projection: QueryReadBacking,
    identity: Weak<QueryReadIdentity>,
    established: Option<PatchStreamPosition>,
    epoch: super::mark_state::DeliveryEpoch,
    dirty: Option<usize>,
    // Weak identity allocation survives until the LAST historical binding drops.
}
impl QueryReadBinding {
    fn belongs_to(&self, coverage: &QueryFactCoverage) -> bool {
        self.identity.ptr_eq(&Arc::downgrade(&coverage.identity))
            && self.projection.belongs_to(&coverage.projection)
    }
}

pub(in crate::domain_computation::primary_graph) enum QueryCoverageStop {
    Native(CompanionPreflightStop),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Verification(super::SettlementVerificationStop),
}
impl From<CompanionPreflightStop> for QueryCoverageStop {
    fn from(value: CompanionPreflightStop) -> Self {
        Self::Native(value)
    }
}
impl From<worth_execution::ExecutionAllocationDenial> for QueryCoverageStop {
    fn from(value: worth_execution::ExecutionAllocationDenial) -> Self {
        Self::Allocation(value)
    }
}
impl From<super::SettlementVerificationStop> for QueryCoverageStop {
    fn from(value: super::SettlementVerificationStop) -> Self {
        Self::Verification(value)
    }
}

/// Only exact native selected-image lookup can issue this ephemeral consequence.
pub(in crate::domain_computation::primary_graph) struct CleanQueryFacts<'selected> {
    _coverage: &'selected QueryFactCoverage,
    _selected: &'selected PositionedRelationalSnapshot,
}

impl SourceInvalidationOwner {
    /// Original facts were compared at this exact query image before entry.
    /// Unsupported projection or a historical capture retains full verification.
    pub(in crate::domain_computation::primary_graph) fn retain_query_fact_coverage(
        self: &Arc<Self>,
        basis: PositionedRelationalSnapshot,
        facts: &[WorthQueryApplicationObservedFact],
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<Arc<QueryFactCoverage>>, QueryCoverageStop> {
        policy.check_live()?;
        if basis.runtime_instance_id() != self.runtime_instance_id || facts.is_empty() {
            return Ok(None);
        }
        let Some(cell) = self.cell_for_read(&basis)? else {
            return Ok(None);
        };
        let image = cell.read_image();
        if !registration::is_live(&image, &basis) {
            return Ok(None);
        }
        // Only the native source-footprint kinds this capture owner actually
        // issues qualify. A newly introduced kind keeps funded full verification.
        for fact in facts {
            policy.check_live()?;

            if !matches!(
                fact,
                WorthQueryApplicationObservedFact::SourceEntity { .. }
                    | WorthQueryApplicationObservedFact::SourceFieldRevision { .. }
                    | WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
                    | WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
            ) {
                return Ok(None);
            }
        }
        let mut unsupported = None;
        let projected = PreparedPostingOrdinals::prepare(facts, None, &mut unsupported)?;
        if unsupported.is_some() {
            return Ok(None);
        }
        let keys = projected.into_query_parts();
        policy.check_live()?;
        let coverage = Arc::new(QueryFactCoverage {
            owner: Arc::clone(self),
            basis,
            projection: Arc::new(QueryReadProjection { keys }),
            identity: Arc::new(QueryReadIdentity),
            slot: Mutex::new(None),
        });
        self.install_query_read(&coverage, &coverage.basis, cell, image, policy)?;
        policy.check_live()?;
        Ok(Some(coverage))
    }

    /// Lookup only the selected image and its exact opaque read binding. Every
    /// intervening publication maintained that binding through the ONE index;
    /// no original-basis retention or touch-history replay is needed.
    pub(in crate::domain_computation::primary_graph) fn query_facts_clean_at<'selected>(
        &self,
        coverage: &'selected QueryFactCoverage,
        selected: &'selected PositionedRelationalSnapshot,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<CleanQueryFacts<'selected>>, QueryCoverageStop> {
        policy.check_live()?;
        if !registration::same_source(self, coverage, selected) {
            return Ok(None);
        }
        let slot = *coverage
            .slot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(slot) = slot else {
            return Ok(None);
        };
        let Some(cell) = self.cell_for_read(selected)? else {
            return Ok(None);
        };
        let image = cell.read_image();

        let Ok(aligned) = SnapshotAlignedMarkState::observe_image(&image, selected) else {
            return Ok(None);
        };
        let state = aligned.recovery_state();

        let Some(super::settlement::PostingBindings::QueryRead(read)) =
            state.posting_bindings.get(&slot).map(Arc::as_ref)
        else {
            return Ok(None);
        };
        if !read.belongs_to(coverage)
            || read.epoch != state.delivery_epoch
            || read.established > selected.position()
            || read.dirty.is_some()
        {
            return Ok(None);
        }
        policy.check_live()?;
        Ok(Some(CleanQueryFacts {
            _coverage: coverage,
            _selected: selected,
        }))
    }
}

impl Drop for QueryFactCoverage {
    fn drop(&mut self) {
        self.owner.release_query_read(self);
    }
}
