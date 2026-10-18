//! Retained computation changes use the same source-aligned native mark image.
use super::super::{RecordedSettlementIdentity, RetainedSourceFacts};
use super::{
    admission::IndexAdmission, source_alignment::SnapshotAlignedMarkState,
    InvalidationEditAdmission, SourceInvalidationOwner,
};
mod postcondition_carry;
use im::OrdSet;
pub(in crate::domain_computation::primary_graph) use postcondition_carry::PostconditionCarry;
use std::sync::Arc;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

/// Only a performed record that actually retained a computation requests tracking.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct ComputationRegistration(());
impl ComputationRegistration {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn retained() -> Self {
        Self(())
    }
}
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ComputationChanges {
    pub(in crate::domain_computation::primary_graph::output_lineage) identity:
        Arc<RecordedSettlementIdentity>,
    selected: worth_relational::facade::runtime::PositionedRelationalSnapshot,
    _image: Arc<super::source_alignment::BranchMarkRoot>,
    ordinals: OrdSet<usize>,
    original: Option<
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    >,
    postconditions:
        crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    snapshot: SnapshotHandle,
    payload_bytes: u64,
    postings: im::OrdMap<Arc<super::fact_key::FactPostingKey>, OrdSet<usize>>,
}
impl ComputationChanges {
    /// A publication can readmit after computation. Only the native owner can
    /// establish that its genuine before-image still names this captured source.
    pub(in crate::domain_computation::primary_graph) fn matches_publication_source(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        if snapshot.runtime_instance_id() != self.selected.runtime_instance_id()
            || snapshot.branch_id() != self.selected.branch_id()
        {
            return Ok(false);
        }
        let actual = runtime
            .read_truth()
            .positioned_snapshot_admitted(snapshot, |work, bytes| {
                admission.work(work)?;
                admission.bytes(bytes)
            });
        use worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop as Stop;
        match actual {
            Ok(actual) => Ok(actual == self.selected),
            Err(Stop::Admission(stop)) => Err(stop),
            Err(Stop::AccountingOverflow) => Err(CompanionPreflightStop::WorkCounterOverflow),
            Err(Stop::Position(_)) => Ok(false),
        }
    }
    /// The same native-position check lends this image to the reader's exact
    /// live pin. The cutoff's separate snapshot handle is never substituted.
    pub(in crate::domain_computation::primary_graph) fn for_snapshot(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Option<Self> {
        self.at_snapshot(runtime, snapshot)?;
        let mut selected = self.clone();
        selected.snapshot = snapshot.clone();
        Some(selected)
    }
    pub(in crate::domain_computation::primary_graph) fn ordinals(&self) -> &OrdSet<usize> {
        &self.ordinals
    }
    pub(in crate::domain_computation::primary_graph::output_lineage) fn bind_original(
        mut self,
        original: crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
    ) -> Self {
        self.original = Some(original);
        self
    }
    pub(in crate::domain_computation::primary_graph) fn at_snapshot(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Option<&OrdSet<usize>> {
        if snapshot.runtime_instance_id() != self.selected.runtime_instance_id()
            || snapshot.branch_id() != self.selected.branch_id()
        {
            return None;
        }
        let actual = runtime.read_truth().positioned_snapshot(snapshot).ok()?;
        (self.selected == actual).then_some(&self.ordinals)
    }
}
impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn computation_changes(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        identity: &Arc<RecordedSettlementIdentity>,
        facts: &RetainedSourceFacts,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ComputationChanges>, CompanionPreflightStop> {
        let selected = runtime
            .read_truth()
            .positioned_snapshot_admitted(snapshot, |work, bytes| {
                admission.work(work)?;
                admission.bytes(bytes)
            })
            .map_err(|stop| match stop {
                worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop::Admission(stop) => stop,
                worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop::AccountingOverflow => CompanionPreflightStop::WorkCounterOverflow,
                worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop::Position(_) => CompanionPreflightStop::SelectedSourceMismatch,
            })?;
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        let Some(cell) = self.cell_for_read(&selected, admission)? else {
            return Ok(None);
        };
        let image = cell.read_image();
        let retained_image = Arc::clone(image.payload());
        admission.ordered_read(image.payload().past.len())?;
        let Ok(aligned) = SnapshotAlignedMarkState::select_image(image, &selected) else {
            return Ok(None);
        };
        admission.ordered_read(aligned.settlement_count())?;
        let Some(comparable) = facts.for_comparison() else {
            return Ok(None);
        };
        let Some(ordinals) = aligned.computation_frontier(identity, facts) else {
            return Ok(None);
        };
        let Some(payload_bytes) = aligned.computation_payload_bytes(identity) else {
            return Ok(None);
        };
        let postings = aligned
            .computation_postings(identity, facts)
            .expect("same aligned row");
        // Prepay the reader's bounded native-position identity check, beside
        // the capture's admitted lookup. No fact population is visited there.
        admission.work(10 + 5 * snapshot.branch_id().0.len() as u64)?;
        admission.bytes(
            size_of::<ComputationChanges>() as u64 + 6 * snapshot.branch_id().0.len() as u64,
        )?;
        Ok(Some(ComputationChanges {
            identity: Arc::clone(identity),
            selected,
            _image: retained_image,
            ordinals,
            original: None,
            postconditions: comparable.facts().clone(),
            snapshot: snapshot.clone(),
            payload_bytes,
            postings,
        }))
    }
}
