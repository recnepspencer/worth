//! Same-registry checkpoint DATA captured before growth under the caller's
//! ORIGINAL configuration read epoch and actual publication exclusion guard.
//! A returned selection value alone does not keep that guard alive.
mod data;
pub(crate) use data::{CheckpointBranchMetadata, CheckpointBranchMetadataFields};

use super::RelationalBranchReferenceRegistry;
use crate::branch::reference::CheckpointCellSource;
use crate::branch::RelationalBranchCellCheckpoint;
use crate::durability::data::DurableBranchLifecycleImage;
use crate::history::data::BranchId;
use worth_execution::ExecutionRequest;

#[derive(Clone, Copy, Debug)]
pub(crate) enum CheckpointBranchMetadataCaptureDenial {
    SizeOverflow,
    SourceChanged,
    Request(crate::execution::RelationalExecutionDenialCause),
}

struct CellSource {
    source: CheckpointCellSource,
    name_bytes: usize,
    index: usize,
}

impl RelationalBranchReferenceRegistry {
    pub(crate) fn capture_checkpoint_metadata(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime_name: &str,
    ) -> Result<CheckpointBranchMetadata, CheckpointBranchMetadataCaptureDenial> {
        use CheckpointBranchMetadataCaptureDenial as Denial;
        let state = self.read();
        let count = state.cells.len();
        let retired_count = state.retired_names.len();
        let scratch =
            std::alloc::Layout::array::<CellSource>(count).map_err(|_| Denial::SizeOverflow)?;
        let scratch_bytes = u64::try_from(scratch.size()).map_err(|_| Denial::SizeOverflow)?;
        let _scratch_memory = request
            .reserve_retained_memory(scratch_bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut sources = Vec::with_capacity(count);
        let mut bytes = data::control_bytes().ok_or(Denial::SizeOverflow)?;
        let mut add = |value: usize| -> Result<(), Denial> {
            bytes = bytes.checked_add(value).ok_or(Denial::SizeOverflow)?;
            Ok(())
        };
        add(runtime_name.len())?;
        add(
            std::alloc::Layout::array::<RelationalBranchCellCheckpoint>(count)
                .map_err(|_| Denial::SizeOverflow)?
                .size(),
        )?;
        add(
            std::alloc::Layout::array::<DurableBranchLifecycleImage>(count)
                .map_err(|_| Denial::SizeOverflow)?
                .size(),
        )?;
        add(std::alloc::Layout::array::<BranchId>(retired_count)
            .map_err(|_| Denial::SizeOverflow)?
            .size())?;
        for (index, (name, cell)) in state.cells.iter().enumerate() {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            // One raw name in the cell DTO and another in its lifecycle DTO.
            add(name.0.len().checked_mul(2).ok_or(Denial::SizeOverflow)?)?;
            sources.push(CellSource {
                source: cell.checkpoint_source(),
                name_bytes: name.0.len(),
                index,
            });
        }
        for name in &state.retired_names {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            add(name.0.len())?;
        }
        drop(add);
        // Never hold the registry while waiting for a branch state lock.
        drop(state);
        for entry in &sources {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            bytes = bytes
                .checked_add(
                    entry
                        .source
                        .nested_allocation_bytes()
                        .ok_or(Denial::SizeOverflow)?,
                )
                .ok_or(Denial::SizeOverflow)?;
        }
        let memory = request
            .reserve_retained_memory(u64::try_from(bytes).map_err(|_| Denial::SizeOverflow)?)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut cells = Vec::with_capacity(count);
        let mut lifecycle = Vec::with_capacity(count);
        let mut retired = Vec::with_capacity(retired_count);
        for entry in &sources {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            let name = {
                let state = self.read();
                let (name, cell) = state
                    .cells
                    .get_index(entry.index)
                    .ok_or(Denial::SourceChanged)?;
                if state.cells.len() != count
                    || name.0.len() != entry.name_bytes
                    || !entry.source.matches(cell)
                {
                    return Err(Denial::SourceChanged);
                }
                name.clone()
            };
            let (cell, posture) = entry.source.copy(name);
            cells.push(cell);
            lifecycle.push(posture);
        }
        {
            let state = self.read();
            if state.retired_names.len() != retired_count {
                return Err(Denial::SourceChanged);
            }
            for name in &state.retired_names {
                request
                    .consult()
                    .map_err(|cause| Denial::Request(cause.into()))?;
                retired.push(name.clone());
            }
        }
        cells.sort_unstable_by(|left, right| left.branch_id.cmp(&right.branch_id));
        lifecycle.sort_unstable_by(|left, right| left.branch_id.cmp(&right.branch_id));
        retired.sort_unstable();
        request
            .consult()
            .map_err(|cause| Denial::Request(cause.into()))?;
        Ok(CheckpointBranchMetadata::captured(
            runtime_name.to_owned(),
            cells,
            lifecycle,
            retired,
            memory,
        ))
    }
}

#[cfg(test)]
mod tests;
