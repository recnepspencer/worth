//! An ordinary recovery read of authentic equality links and terminal facts.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::mark_state::MarkState;
use super::mark_state::OutputFactCoverage;
use super::output_facts::RegisteredOutputFacts;
use super::semantic_fence::{observe, same_observed_meaning, ObservedRows};
use super::source_alignment::BranchMarkRoot;
use super::source_alignment::SnapshotAlignedMarkState;
use super::SettlementVerificationStop;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

/// One aligned read of the mark image a walk follows equality links in.
/// Cloning it shares the same read.
#[derive(Clone)]
pub struct EqualityRecoveryImage {
    cell: CompanionBranchCell<BranchMarkRoot>,
    payload: Arc<BranchMarkRoot>,
    selected: PositionedRelationalSnapshot,
    state: Arc<MarkState>,
}

pub struct EqualityRecoveryRow {
    _custody: Arc<super::mark_state::SettlementMarks>,
    pub identity: Arc<RecordedSettlementIdentity>,
    pub facts: super::super::RetainedSourceFacts,
    output_projection: Option<RegisteredOutputFacts>,
    pub upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub read_basis: Arc<PositionedRelationalSnapshot>,
    /// The mark row's coordinate for its source `facts` (public #358).
    pub historical: Option<super::super::SelectionCoordinate>,
}

impl EqualityRecoveryRow {
    /// The coordinate the output projection carries with its facts.
    pub fn output_historical(&self) -> Option<&super::super::SelectionCoordinate> {
        self.output_projection.as_ref()?.historical.as_ref()
    }

    pub fn output_facts(&self) -> Option<&[WorthQueryApplicationObservedFact]> {
        self.output_projection
            .as_ref()
            .map(|projection| projection.facts.as_ref())
    }
}

impl SourceInvalidationOwner {
    pub fn equality_recovery_image(
        &self,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<EqualityRecoveryImage, SettlementVerificationStop> {
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let cell = self
            .cell_for_read(selected)?
            .ok_or(SettlementVerificationStop::Alignment)?;
        let image = cell.read_image();

        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        Ok(EqualityRecoveryImage {
            state: aligned.recovery_state(),
            payload: Arc::clone(image.payload()),
            cell,
            selected: selected.clone(),
        })
    }
}

impl EqualityRecoveryImage {
    /// The same aligned read, with its state for a semantic comparison.
    pub fn state(&self) -> &MarkState {
        &self.state
    }

    /// Whether `other` is this same read (the payload is shared).
    pub fn same_read(&self, other: &Self) -> bool {
        Arc::ptr_eq(&other.payload, &self.payload)
            && Arc::ptr_eq(&other.state, &self.state)
    }

    /// Resolve `identity` to its terminal row. `observed` gains the identity
    /// asked for, the terminal, and every row the projection walk passes,
    /// as the oracle records them (`full_verification.rs`).
    pub fn terminal_row(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
        observed: &mut ObservedRows,
    ) -> Result<EqualityRecoveryRow, SettlementVerificationStop> {
        observe(observed, identity);
        let mut current = Arc::clone(identity);
        for _ in 0..=self.state.equal_links.len() {
            let next = self
                .state
                .equal_links
                .get(&current)
                .and_then(|link| link.next.as_ref());
            let Some(next) = next else {
                let row = self
                    .state
                    .settlements
                    .get(&current)
                    .ok_or(SettlementVerificationStop::Alignment)?;
                observe(observed, &current);
                return Ok(EqualityRecoveryRow {
                    _custody: Arc::clone(row),
                    output_projection: self.output_facts(&current, observed)?,
                    identity: current,
                    facts: row.facts.clone(),
                    upstream: row.consumed_upstream.clone(),
                    read_basis: Arc::clone(&row.read_basis),
                    historical: row.historical.clone(),
                });
            };

            if self
                .state
                .equal_links
                .get(next)
                .is_none_or(|following| following.prior.as_ref() != Some(&current))
            {
                return Err(SettlementVerificationStop::Alignment);
            }
            current = Arc::clone(next);
        }
        Err(SettlementVerificationStop::Alignment)
    }

    fn output_facts(
        &self,
        terminal: &Arc<RecordedSettlementIdentity>,
        observed: &mut ObservedRows,
    ) -> Result<Option<RegisteredOutputFacts>, SettlementVerificationStop> {
        let mut current = terminal;
        for _ in 0..=self.state.equal_links.len() {
            observe(observed, current);
            let Some(row) = self.state.settlements.get(current) else {
                return Ok(None);
            };
            if let Some(output) = &row.output_facts {
                return Ok(Some(output.clone()));
            }
            if row.output_coverage != OutputFactCoverage::Stable {
                return Ok(None);
            }

            let prior = self
                .state
                .equal_links
                .get(current)
                .and_then(|link| link.prior.as_ref())
                .ok_or(SettlementVerificationStop::Alignment)?;

            if self
                .state
                .equal_links
                .get(prior)
                .is_none_or(|preceding| preceding.next.as_ref() != Some(current))
            {
                return Err(SettlementVerificationStop::Alignment);
            }
            current = prior;
        }
        Err(SettlementVerificationStop::Alignment)
    }

    /// Comparisons precede any recovery edit. The fence trips when a row the
    /// walk read (`observed`) changed meaning since this read
    /// (`semantic_fence`), or when `selected` no longer aligns. Mark-only
    /// carries by the walk's own nested verifications pass.
    pub fn fence(&self, observed: &ObservedRows) -> Result<(), SettlementVerificationStop> {
        let fresh = self.cell.read_image();
        if Arc::ptr_eq(fresh.payload(), &self.payload) {
            return Ok(());
        }
        let stop = || SettlementVerificationStop::Edit(CompanionCellEditStop::TopologyGenerationChanged);
        let aligned =
            SnapshotAlignedMarkState::observe_image(&fresh, &self.selected).map_err(|_| stop())?;
        if !same_observed_meaning(&self.state, &aligned.recovery_state(), observed, &self.selected) {
            return Err(stop());
        }
        Ok(())
    }
}
