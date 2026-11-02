//! The original capture owns each covered cell and its bounded transfer cursor data.
use crate::durability::data::{
    RelationalCheckpointDirectory, RelationalCheckpointManifestDraft,
    RelationalNativeCheckpointCaptureDenial as Capture,
};
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use crate::history::{
    RelationalCommitEnvelopeResidency, RetainedCanonicalIndex, RetiredCanonicalPlacement,
};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
    MapKernelContext,
};
#[derive(Debug)]
struct CapturedCell {
    entry: RetainedCanonicalIndex,
    cell: Arc<RelationalCommitEnvelopeResidency>,
}
#[derive(Debug)]
struct Backing {
    cells: ExecutionArray<CapturedCell>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Clone, Debug)]
pub(crate) struct CanonicalCheckpointPublication(Arc<Backing>);
impl CanonicalCheckpointPublication {
    pub(crate) fn capture(
        manifest: &RelationalCheckpointManifestDraft,
        cells: &worth_execution::ExecutionOrderedMap<
            crate::publication::patch::data::PatchStreamPosition,
            Arc<RelationalCommitEnvelopeResidency>,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Capture> {
        let bytes = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| {
                Capture::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Capture::Request(cause.into()))?;
        let mut rows = ExecutionRequestArrayBuilder::allocate(*request, manifest.commits.len())
            .map_err(super::capture::array_capture_stop)?;
        for original in &manifest.commits {
            let cell = cells.get(&original.position).ok_or_else(|| {
                Capture::Durability(crate::durability::data::DurabilityError::new(
                    crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                    "captured canonical cell unavailable",
                ))
            })?;
            let entry = match cell.source_placement() {
                Some((entry, _)) if entry.as_ref() == original => entry,
                Some(_) => {
                    return Err(Capture::Durability(
                        crate::durability::data::DurabilityError::new(
                            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
                            "captured canonical entry differs",
                        ),
                    ))
                }
                None => RetainedCanonicalIndex::copy_original(original, request)
                    .map_err(Capture::Request)?,
            };
            rows.push(CapturedCell {
                entry,
                cell: cell.clone(),
            })
            .map_err(super::capture::array_capture_stop)?;
        }
        let cells = rows.seal().map_err(super::capture::array_capture_stop)?;
        Ok(Self(Arc::new(Backing {
            cells,
            _memory: memory,
        })))
    }
    pub(crate) fn len(&self) -> usize {
        self.0.cells.len()
    }
    pub(crate) fn advance(
        &self,
        next: &mut usize,
        retired: &mut RetiredCanonicalPlacements,
        directory: &Arc<RelationalCheckpointDirectory>,
        request: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Ack> {
        while let Some(row) = self.0.cells.get(*next) {
            request.check_live().map_err(Ack::Stopped)?;
            if context.has_accounting() {
                context.checkpoint(1).map_err(Ack::Stopped)?;
            } else {
                context.poll().map_err(Ack::Stopped)?;
            }
            let slot = retired
                .slots
                .get_mut(*next)
                .expect("capture pre-admitted exact covered placement slots");
            if slot.backing.is_none() {
                let old = row
                    .cell
                    .install_selected(row.entry.clone(), directory.clone())?;
                slot.backing = Some(old);
            }
            *next += 1;
        }
        Ok(())
    }
}

/// One small slot per covered cell, plus the displaced selection pointer.
#[derive(Debug, Default)]
struct DisplacedSlot {
    backing: Option<RetiredCanonicalPlacement>,
    directory: Option<Arc<RelationalCheckpointDirectory>>,
    inert: Option<crate::durability::data::CheckpointDirectoryRetirement>,
}
/// Unique operation storage, mutated only under CheckpointPublication's phase lock.
#[derive(Debug)]
pub(crate) struct RetiredCanonicalPlacements {
    slots: Vec<DisplacedSlot>,
    next: usize,
    active: Option<crate::durability::data::CheckpointDirectoryRetirement>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RetiredCanonicalPlacements {
    pub(crate) fn prepare(
        request: &ExecutionRequest<'_, '_>,
        count: usize,
    ) -> Result<Self, Capture> {
        let count = count.checked_add(1).ok_or(Capture::Request(
            crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
        ))?;
        let bytes = Layout::array::<DisplacedSlot>(count)
            .map_err(|_| {
                Capture::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
            })?
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Capture::Request(cause.into()))?;
        let mut slots = Vec::with_capacity(count);
        slots.resize_with(count, DisplacedSlot::default);
        Ok(Self {
            slots,
            next: 0,
            active: None,
            _memory: memory,
        })
    }
    pub(crate) fn displaced_selection(
        &mut self,
        directory: Option<Arc<RelationalCheckpointDirectory>>,
    ) {
        let slot = self
            .slots
            .last_mut()
            .expect("the operation pre-admitted its displaced selection pointer");
        debug_assert!(slot.directory.is_none());
        slot.directory = directory;
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Ack> {
        while self.next < self.slots.len() {
            request.check_live().map_err(Ack::Stopped)?;
            if context.has_accounting() {
                context.checkpoint(1).map_err(Ack::Stopped)?;
            } else {
                context.poll().map_err(Ack::Stopped)?;
            }
            if let Some(task) = &mut self.active {
                if !task.advance(request)? {
                    continue;
                }
                self.slots[self.next].inert = self.active.take();
                self.next += 1;
                continue;
            }
            let slot = &self.slots[self.next];
            let has_directory = slot.directory.is_some()
                || slot
                    .backing
                    .as_ref()
                    .is_some_and(|backing| backing.has_directory());
            if !has_directory {
                self.next += 1;
                continue;
            }
            // Each original directory carries its own prepaid retirement storage.
            let slot = &mut self.slots[self.next];
            let directory = slot
                .directory
                .take()
                .or_else(|| {
                    slot.backing
                        .as_mut()
                        .and_then(|backing| backing.take_directory())
                })
                .expect("checked displaced directory is retained");
            if let Some(directory) = Arc::into_inner(directory) {
                self.active = Some(directory.into_retirement());
            } else {
                // Arc::into_inner atomically released only this genuine nonfinal share.
                self.next += 1;
            }
        }
        Ok(())
    }
}
