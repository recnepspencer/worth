//! One captured publication roster owns its exact pointer/header allocation.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::storage::residency::RelationalCheckpointRegionPublication;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    rows: Vec<RelationalCheckpointRegionPublication>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Clone, Default)]
pub(super) struct CapturedRegionPublications(Option<Arc<Backing>>);
impl CapturedRegionPublications {
    pub(super) fn entries(&self) -> &[RelationalCheckpointRegionPublication] {
        self.0.as_ref().map_or(&[], |owner| &owner.rows)
    }
    pub(super) fn from_ordinary(rows: Vec<RelationalCheckpointRegionPublication>) -> Self {
        if rows.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            rows,
            _memory: None,
        })))
    }
    pub(super) fn capture_for_request(
        count: usize,
        rows: impl IntoIterator<Item = RelationalCheckpointRegionPublication>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        if count == 0 {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            return Ok(Self::default());
        }
        let control = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Backing>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size());
        let bytes = control
            .and_then(|control| {
                std::alloc::Layout::array::<RelationalCheckpointRegionPublication>(count)
                    .ok()
                    .and_then(|rows| control.checked_add(rows.size()))
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Request(
                crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
            ))?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Denial::Request(cause.into()))?;
        let mut values = Vec::with_capacity(count);
        for row in rows {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            if values.len() == count {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            values.push(row);
        }
        if values.len() != count {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(Self(Some(Arc::new(Backing {
            rows: values,
            _memory: memory,
        }))))
    }
}
