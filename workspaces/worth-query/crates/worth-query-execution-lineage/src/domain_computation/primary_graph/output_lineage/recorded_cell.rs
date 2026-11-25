//! Type-erased shared handle to one `OnceLock<RecordedOutput>` cell.
//!
//! Uses unsized coercion on the existing `Arc<OnceLock<RecordedOutput>>`
//! allocation so rustdoc auto-trait synthesis does not recursively walk
//! `RecordedOutput` and its self-referential `performed_origin` chain through
//! settlement and observation handles.

use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use super::RecordedOutput;

trait RecordedOutputCellBacking: Send + Sync {
    fn as_cell(&self) -> &OnceLock<RecordedOutput>;
}

impl RecordedOutputCellBacking for OnceLock<RecordedOutput> {
    fn as_cell(&self) -> &OnceLock<RecordedOutput> {
        self
    }
}

#[derive(Clone)]
pub(super) struct SharedRecordedOutputCell(Arc<dyn RecordedOutputCellBacking>);

impl SharedRecordedOutputCell {
    pub(super) fn from_arc(cell: Arc<OnceLock<RecordedOutput>>) -> Self {
        Self(cell)
    }

    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(&**self, &**other)
    }
}

impl Deref for SharedRecordedOutputCell {
    type Target = OnceLock<RecordedOutput>;

    fn deref(&self) -> &Self::Target {
        RecordedOutputCellBacking::as_cell(&*self.0)
    }
}
