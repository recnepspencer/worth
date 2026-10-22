use crate::ExecutionRetainedResult;
use worth_foundational::PartitionIdentity;

/// Exact native costs, never application identity or permission to carry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunSummary {
    pub(crate) accounted: bool,
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
            accounted: true,
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
        let cost = value.accounted_cost();
        Self {
            accounted: cost.is_some(),
            count: 1,
            work: cost.map_or(0, |cost| cost.0),
            span: cost.map_or(0, |cost| cost.1),
            maximum_slot_work: cost.map_or(0, |cost| cost.0),
            first: Some(value.identity()),
            last: Some(value.identity()),
            unqualified: (!value.native_reuse_available()).then_some(value.identity()),
        }
    }
    pub(super) fn append(self, right: Self) -> Option<Self> {
        Some(Self {
            accounted: self.accounted && right.accounted,
            count: self.count.checked_add(right.count)?,
            work: if self.accounted && right.accounted {
                self.work.checked_add(right.work)?
            } else {
                0
            },
            span: if self.accounted && right.accounted {
                self.span.max(right.span)
            } else {
                0
            },
            maximum_slot_work: if self.accounted && right.accounted {
                self.maximum_slot_work.max(right.maximum_slot_work)
            } else {
                0
            },
            first: self.first.or(right.first),
            last: right.last.or(self.last),
            unqualified: self.unqualified.or(right.unqualified),
        })
    }
}
