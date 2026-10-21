//! An ordinary recovery read of authentic equality links and terminal facts.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionBranchImage;
use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::mark_state::MarkState;
use super::mark_state::OutputFactCoverage;
use super::output_facts::RegisteredOutputFacts;
use super::source_alignment::BranchMarkRoot;
use super::source_alignment::SnapshotAlignedMarkState;
use super::SettlementVerificationStop;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

pub(in crate::domain_computation::primary_graph) struct EqualityRecoveryImage {
    cell: CompanionBranchCell<BranchMarkRoot>,
    image: CompanionBranchImage<BranchMarkRoot>,
    state: Arc<MarkState>,
}

pub(in crate::domain_computation::primary_graph) struct EqualityRecoveryRow {
    _custody: Arc<super::mark_state::SettlementMarks>,
    pub identity: Arc<RecordedSettlementIdentity>,
    pub facts: super::super::RetainedSourceFacts,
    output_projection: Option<RegisteredOutputFacts>,
    pub upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub read_basis: Arc<PositionedRelationalSnapshot>,
}

impl EqualityRecoveryRow {
    pub(in crate::domain_computation::primary_graph) fn output_facts(
        &self,
    ) -> Option<&[WorthQueryApplicationObservedFact]> {
        self.output_projection
            .as_ref()
            .map(|projection| projection.facts.as_ref())
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn equality_recovery_image(
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
            cell,
            image,
            state: aligned.recovery_state(),
        })
    }
}

impl EqualityRecoveryImage {
    pub(in crate::domain_computation::primary_graph) fn terminal_row(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<EqualityRecoveryRow, SettlementVerificationStop> {
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
                return Ok(EqualityRecoveryRow {
                    _custody: Arc::clone(row),
                    output_projection: self.output_facts(&current)?,
                    identity: current,
                    facts: row.facts.clone(),
                    upstream: row.consumed_upstream.clone(),
                    read_basis: Arc::clone(&row.read_basis),
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
    ) -> Result<Option<RegisteredOutputFacts>, SettlementVerificationStop> {
        let mut current = terminal;
        for _ in 0..=self.state.equal_links.len() {
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

    /// Comparisons precede any recovery edit. A same-position actor edit
    /// invalidates this read just as a native publication does.
    pub(in crate::domain_computation::primary_graph) fn fence(
        &self,
    ) -> Result<(), SettlementVerificationStop> {
        let fresh = self.cell.read_image();
        if fresh.root_id() != self.image.root_id()
            || fresh.commit_id() != self.image.commit_id()
            || fresh.position() != self.image.position()
            || fresh.topology_generation() != self.image.topology_generation()
            || !Arc::ptr_eq(fresh.payload(), self.image.payload())
        {
            return Err(SettlementVerificationStop::Edit(
                CompanionCellEditStop::TopologyGenerationChanged,
            ));
        }
        Ok(())
    }
}
