use crate::ExecutionRetainedResult;
use worth_foundational::PartitionIdentity;

/// Exact native costs, never application identity or permission to carry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunSummary {
    pub(crate) count: usize,
    pub(crate) work: u64,
    pub(crate) span: u64,
    pub(crate) maximum_slot_work: u64,
    pub(crate) first: Option<PartitionIdentity>,
    pub(crate) last: Option<PartitionIdentity>,
    pub(crate) unqualified: Option<PartitionIdentity>,
}
impl RunSummary {
    pub(crate) fn empty() -> Self {
        Self {
            count: 0,
            work: 0,
            span: 0,
            maximum_slot_work: 0,
            first: None,
            last: None,
            unqualified: None,
        }
    }
    pub(super) fn slot<R>(value: &ExecutionRetainedResult<R>) -> Self {
        Self {
            count: 1,
            work: value.charged_work(),
            span: value.charged_span(),
            maximum_slot_work: value.charged_work(),
            first: Some(value.identity()),
            last: Some(value.identity()),
            unqualified: (!value.native_reuse_available()).then_some(value.identity()),
        }
    }
    pub(super) fn append(self, right: Self) -> Option<Self> {
        Some(Self {
            count: self.count.checked_add(right.count)?,
            work: self.work.checked_add(right.work)?,
            span: self.span.max(right.span),
            maximum_slot_work: self.maximum_slot_work.max(right.maximum_slot_work),
            first: self.first.or(right.first),
            last: right.last.or(self.last),
            unqualified: self.unqualified.or(right.unqualified),
        })
    }
}
