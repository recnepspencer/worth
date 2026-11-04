//! One opaque shared owner retains the actual Home read through both runtimes.
mod native_anchor;
mod native_prefix;
mod read;
use super::{ApplicationHomeCheckpointDenial as Denial, SelectedHomeManifest};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_store::physical_runtime::*;

#[derive(Clone)]
pub(crate) struct SelectedHomeSource {
    backing: Arc<SelectedHomeSourceBacking>,
}
struct SelectedHomeSourceBacking {
    cut: Mutex<HomeReadCut>,
    _funding: Option<ExecutionMemoryReservation>,
}
struct HomeReadCut {
    reader: Option<PhysicalRecordReader>,
    selected: Option<VerifiedHomeRead>,
}
struct VerifiedHomeRead {
    membership: AnchoredRecordReadMembership,
    bytes: worth_execution::ExecutionImmutableBytes,
}
impl SelectedHomeSource {
    pub(crate) fn capture(
        serving: &ServingPhysicalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<SelectedHomeSourceBacking>())
            .map_err(|_| Denial::SizeLimit)?
            .0
            .pad_to_align();
        let funding = request
            .reserve_retained_memory(u64::try_from(layout.size()).map_err(|_| Denial::SizeLimit)?)
            .map_err(Denial::Execution)?;
        let reader = serving
            .try_records_for_checkpoint(request)
            .map_err(|cause| match cause {
                PhysicalCheckpointReadProtectionDenial::Request(cause) => {
                    Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
                }
                cause => Denial::CheckpointReadProtection(cause),
            })?;
        Ok(Self {
            backing: Arc::new(SelectedHomeSourceBacking {
                cut: Mutex::new(HomeReadCut {
                    reader: Some(reader),
                    selected: None,
                }),
                _funding: funding,
            }),
        })
    }
    pub(crate) fn selected(
        &self,
        serving: &ServingPhysicalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<SelectedHomeManifest>, Denial> {
        let mut cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        read::selected(&mut cut, serving, request)
    }
    pub(crate) fn journal_head(
        &self,
        serving: &ServingPhysicalRuntime,
        name: OrderedJournalName,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Denial> {
        request.consult().map_err(Denial::Execution)?;
        let cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        let journal = serving
            .ordered_journal(name)
            .map_err(Denial::JournalUnavailable)?;
        if let Some(selected) = &cut.selected {
            journal.head_position_from_read(&selected.membership)
        } else {
            journal.head_position_from(cut.reader.as_ref().ok_or(Denial::ReadSourceBusy)?)
        }
        .map_err(Denial::JournalRead)
    }
    pub(crate) fn journal_payload(
        &self,
        serving: &ServingPhysicalRuntime,
        name: OrderedJournalName,
        position: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            OrderedJournalEntry,
            worth_execution::ExecutionImmutableBytes,
        ),
        Denial,
    > {
        let cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        read::journal_payload(&cut, serving, name, position, 64 << 10, request)
    }
    pub(crate) fn native_journal_payload(
        &self,
        serving: &ServingPhysicalRuntime,
        name: OrderedJournalName,
        position: u64,
        limit: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<worth_execution::ExecutionImmutableBytes, Denial> {
        let cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        read::journal_payload(&cut, serving, name, position, limit, request).map(|(_, bytes)| bytes)
    }
    pub(crate) fn native_journal_prefix(
        &self,
        serving: &ServingPhysicalRuntime,
        name: OrderedJournalName,
        position: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 10], Denial> {
        let cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        native_prefix::read(&cut, serving, name, position, request)
    }
    pub(crate) fn native_initial_inputs(
        &self,
        serving: &ServingPhysicalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<worth_execution::ExecutionImmutableBytes>, Denial> {
        let cut = self
            .backing
            .cut
            .try_lock()
            .map_err(|_| Denial::ReadSourceBusy)?;
        native_anchor::read(&cut, serving, request)
    }
}
