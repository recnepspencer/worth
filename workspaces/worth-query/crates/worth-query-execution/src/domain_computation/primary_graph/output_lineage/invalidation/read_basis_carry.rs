//! Marks read at the live source image account for every delivery through
//! it, so a row checked there moves its read basis to that image and keeps
//! its marks. A row checked once per retained window never leaves the window.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::{CompanionBranchCell, CompanionBranchImage, CompanionPreflightStop},
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use super::super::RecordedSettlementIdentity;
use super::{
    admission::IndexAdmission,
    index_capacity,
    mark_state::{MarkState, SettlementCurrentness, SettlementMarks},
    retention,
    source_alignment::{BranchMarkRoot, SnapshotAlignedMarkState},
    verification::SettlementVerificationStop,
    InvalidationEditAdmission, SourceInvalidationOwner,
};

/// Moves one row of `next` to the live image `aligned` observed at `selected`
/// and returns whether the row changed. A row that requires full verification
/// keeps its basis: its marks do not account for the deliveries behind it.
/// Every row carried by one edit shares the one `basis` allocation.
pub(super) fn carry_row(
    next: &mut MarkState,
    aligned: &SnapshotAlignedMarkState<'_>,
    selected: &PositionedRelationalSnapshot,
    basis: &mut Option<Arc<PositionedRelationalSnapshot>>,
    identity: &Arc<RecordedSettlementIdentity>,
    admission: &mut impl IndexAdmission,
) -> Result<bool, CompanionPreflightStop> {
    admission.work(1)?;
    admission.ordered_read(next.settlements.len())?;
    let Some(row) = next.settlements.get(identity) else {
        return Ok(false);
    };
    if *row.read_basis == *selected
        || matches!(
            aligned.currentness(identity),
            SettlementCurrentness::FullVerificationRequired(_)
        )
    {
        return Ok(false);
    }
    let basis = match basis {
        Some(basis) => Arc::clone(basis),
        None => {
            admission.bytes(
                index_capacity::arc_bytes::<PositionedRelationalSnapshot>()
                    .and_then(|bytes| bytes.checked_add(selected.branch_id().0.len() as u64))
                    .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            Arc::clone(basis.insert(Arc::new(selected.clone())))
        }
    };
    admission.bytes(
        index_capacity::arc_bytes::<SettlementMarks>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    let mut replacement = (**row).clone();
    replacement.read_basis = basis;
    admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
        next.settlements.len(),
    )?;
    next.settlements
        .insert(Arc::clone(identity), Arc::new(replacement));
    Ok(true)
}

impl SourceInvalidationOwner {
    /// The caller read the marks of these rows at `selected`. Each one whose
    /// marks are complete there moves its read basis to it in one edit.
    /// Historical images stay immutable, and a racing delivery or registration
    /// returns its typed retry without moving any row.
    pub(in crate::domain_computation::primary_graph) fn carry_read_basis(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identities: &[Arc<RecordedSettlementIdentity>],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), SettlementVerificationStop> {
        if identities.is_empty() {
            return Ok(());
        }
        let branch_bytes = selected.branch_id().0.len() as u64;
        admission.work(1 + branch_bytes)?;
        admission.bytes(branch_bytes)?;
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(());
        };
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return Ok(());
        }
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        admission.bytes(
            index_capacity::arc_bytes::<MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut next = (*image.payload().current).clone();
        let mut basis = None;
        let mut carried = false;
        for identity in identities {
            admission.ordered_read(image.payload().past.len())?;
            carried |= carry_row(
                &mut next, &aligned, selected, &mut basis, identity, admission,
            )?;
        }
        drop(aligned);
        if !carried {
            return Ok(());
        }
        self.install_live_state(cell, image, next, admission)
    }

    /// Replaces the live mark state of the image it was prepared against.
    pub(super) fn install_live_state(
        &self,
        cell: CompanionBranchCell<BranchMarkRoot>,
        image: CompanionBranchImage<BranchMarkRoot>,
        mut next: MarkState,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), SettlementVerificationStop> {
        retention::admit_state(&mut next, &self.resources, admission)?;
        admission.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut root = (**image.payload()).clone();
        root.current = Arc::new(next);
        retention::admit_root(&mut root, &self.resources, admission)?;
        let prepared = self
            .prepare_root_replacement(cell, image, Arc::new(root), admission)
            .map_err(SettlementVerificationStop::from)?;
        let cleanup = prepared
            .install()
            .map_err(|stopped| SettlementVerificationStop::Edit(stopped.reason()))?;
        drop(cleanup);
        Ok(())
    }
}
