//! Atomic retained publication of the ordered lineage history and both indexes.
mod custody;
mod edits;
use super::{DiagnosticHistory, DiagnosticsState};
use crate::data::handle::NodeId;
use crate::data::persistent_ord_map::{
    PersistentOrdMap, RetainedMapMutationDenial, RetainedMapMutationOutcome,
};
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageForkPreparation, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial, SignalConditionalRetentionDenial,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation as Reservation,
};
use crate::diagnostics::lineage::{LineageArtifactId, LineageRecord};
pub(super) use custody::LineageRetentionCustody;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum LineagePublicationDenial {
    Accounting(RetainedStoragePreparationDenial),
    Retention(SignalConditionalRetentionDenial),
    Unprepared,
}
impl From<RetainedStoragePreparationDenial> for LineagePublicationDenial {
    fn from(denial: RetainedStoragePreparationDenial) -> Self {
        Self::Accounting(denial)
    }
}
impl From<SignalConditionalRetentionDenial> for LineagePublicationDenial {
    fn from(denial: SignalConditionalRetentionDenial) -> Self {
        Self::Retention(denial)
    }
}
type Denial = LineagePublicationDenial;

struct LineagePublication {
    records: DiagnosticHistory<LineageRecord>,
    nodes: PersistentOrdMap<NodeId, DiagnosticHistory<LineageRecord>>,
    artifacts: PersistentOrdMap<LineageArtifactId, DiagnosticHistory<LineageRecord>>,
    // Retain both old and new storage through all intermediate publication steps.
    previous: LineageRetentionCustody,
    resources: Reservation,
}

impl DiagnosticsState {
    pub(crate) fn record_retained_lineage(
        &mut self,
        record: LineageRecord,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        work: &mut Work,
    ) -> Result<(), Denial> {
        // Never turn missing writer-maintained facts into an ordinary history scan.
        let limit = self
            .installed_retention_budget
            .history_limit
            .max(1)
            .checked_mul(32)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?;
        let history = self
            .lineage_records
            .prepared_retained_charge()
            .map_err(|_| Denial::Unprepared)?;
        self.lineage_records_by_node
            .prepared_retained_charge()
            .map_err(map_denial)?;
        self.lineage_records_by_artifact
            .prepared_retained_charge()
            .map_err(map_denial)?;
        let nodes = self.lineage_records_by_node.prepare_fork_charge(work)?;
        let artifacts = self.lineage_records_by_artifact.prepare_fork_charge(work)?;
        let retained = history
            .checked_add(nodes.retained)?
            .checked_add(artifacts.retained)?;
        // Bound the complete staged roots, three new frame copies, and replacement
        // tree granules before any clone or insertion. Shared payloads are charged
        // conservatively per root; final publication never repairs these facts.
        let frame = arc_allocation_charge::<LineageRecord>()?
            .checked_add(record.retained_heap_charge(work)?)?;
        let extent = self
            .lineage_records
            .len()
            .checked_add(1)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?;
        let growth = frame
            .checked_mul(3)?
            .checked_add(ordered_index_charge::<u64, Arc<LineageRecord>>(extent)?.checked_mul(3)?)?
            .checked_add(ordered_index_charge::<
                Arc<NodeId>,
                Arc<DiagnosticHistory<LineageRecord>>,
            >(self.lineage_records_by_node.len() + 1)?)?
            .checked_add(ordered_index_charge::<
                Arc<LineageArtifactId>,
                Arc<DiagnosticHistory<LineageRecord>>,
            >(self.lineage_records_by_artifact.len() + 1)?)?
            .checked_add(arc_allocation_charge::<NodeId>()?)?
            .checked_add(arc_allocation_charge::<LineageArtifactId>()?)?
            .checked_add(
                arc_allocation_charge::<DiagnosticHistory<LineageRecord>>()?.checked_mul(2)?,
            )?;
        let maximum = retained.checked_mul(2)?.checked_add(growth)?;
        let total = maximum
            .checked_add(nodes.source_growth)?
            .checked_add(artifacts.source_growth)?
            .checked_add(Charge::capacity::<LineagePublication>(1)?)?
            .checked_add(arc_allocation_charge::<Reservation>()?)?;
        work.reserve_visits(std::mem::size_of::<LineagePublication>())?;
        let mut resources = ledger.reserve(0, total)?;
        let mut staged = LineagePublication {
            records: self.lineage_records.clone(),
            nodes: self.lineage_records_by_node.fork_reserved(&mut resources),
            artifacts: self
                .lineage_records_by_artifact
                .fork_reserved(&mut resources),
            previous: self.lineage_custody.clone(),
            resources,
        };
        staged.append(record, maximum, work)?;
        staged.evict_to(limit, work)?;
        let required = staged
            .records
            .prepared_retained_charge()
            .map_err(|_| Denial::Unprepared)?
            .checked_add(
                staged
                    .nodes
                    .prepared_retained_charge()
                    .map_err(map_denial)?,
            )?
            .checked_add(
                staged
                    .artifacts
                    .prepared_retained_charge()
                    .map_err(map_denial)?,
            )?;
        if required > maximum {
            return Err(Denial::Retention(
                SignalConditionalRetentionDenial::CapacityExhausted,
            ));
        }
        let LineagePublication {
            records,
            nodes,
            artifacts,
            previous,
            resources,
        } = staged;
        self.lineage_records = records;
        self.lineage_records_by_node = nodes;
        self.lineage_records_by_artifact = artifacts;
        self.lineage_custody = LineageRetentionCustody(Some(Arc::new(resources)));
        drop(previous);
        Ok(())
    }
}
fn map_denial(denial: RetainedMapMutationDenial) -> Denial {
    match denial {
        RetainedMapMutationDenial::Accounting(denial) => Denial::Accounting(denial),
        _ => Denial::Unprepared,
    }
}
fn accounted<R>(
    outcome: Result<RetainedMapMutationOutcome<R>, RetainedMapMutationDenial>,
) -> Result<R, Denial> {
    match outcome.map_err(map_denial)? {
        RetainedMapMutationOutcome::Accounted { output, .. } => Ok(output),
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => Err(Denial::Accounting(denial)),
    }
}

#[cfg(test)]
mod tests;
