//! Batch rank windows split at actual discriminating bits, never empty bit levels.
use super::*;
use std::ops::Range;

pub(super) struct Batch<'pages, 'control, 'call, 'request, Node> {
    pub(super) pages: &'pages [(usize, &'pages WorkingPage<'request, Node>)],
    pub(super) execution: ExecutionRequest<'request, 'request>,
    pub(super) control: &'control WorthQueryManagedIndexControl<'call>,
    pub(super) allocations: &'control Cell<usize>,
}
impl<Node: Clone> Batch<'_, '_, '_, '_, Node> {
    fn key_at(&self, rank: usize) -> Result<usize, Stop> {
        self.control.checkpoint(size_of::<usize>() as u64)?;
        Ok(self.pages[rank].0)
    }
    pub(super) fn subtree(
        &self,
        previous: Option<Arc<Directory<Node>>>,
        window: Range<usize>,
    ) -> Result<Option<Arc<Directory<Node>>>, Stop> {
        self.control.checkpoint(1)?;
        if window.is_empty() {
            return Ok(previous);
        }
        let mut first = self.key_at(window.start)?;
        let mut last = self.key_at(window.end - 1)?;
        if let Some(old) = &previous {
            first = first.min(old.prefix);
            last = last.max(old.prefix | ((1 << old.height) - 1));
        }
        let height = usize::BITS - (first ^ last).leading_zeros();
        if height == 0 {
            debug_assert_eq!(window.len(), 1);
            return self.page(window.start, first);
        }
        let split = (first & (usize::MAX << height)) | (1 << (height - 1));
        let (left, right) = match previous {
            Some(old) if old.height == height => match &old.entry {
                Entry::Branch { left, right } => (Some(left.clone()), Some(right.clone())),
                Entry::Page(_) => unreachable!("nonzero branch height"),
            },
            Some(old) if old.prefix < split => (Some(old), None),
            old => (None, old),
        };
        let mut lower = window.start;
        let mut upper = window.end;
        while lower < upper {
            let middle = lower + (upper - lower) / 2;
            if self.key_at(middle)? < split {
                lower = middle + 1;
            } else {
                upper = middle;
            }
        }
        let left = self.subtree(left, window.start..lower)?;
        let right = self.subtree(right, lower..window.end)?;
        let allocated = left.is_some() && right.is_some();
        let result = branch(&self.execution, left, right, self.control)?;
        if allocated && result.is_some() {
            self.allocations.set(self.allocations.get() + 1);
        }
        Ok(result)
    }
    fn page(&self, rank: usize, key: usize) -> Result<Option<Arc<Directory<Node>>>, Stop> {
        self.control
            .checkpoint(size_of_val(&self.pages[rank]) as u64)?;
        let page = self.pages[rank].1;
        let (builder, occupied) = {
            let mut open = page.open.borrow_mut();
            (
                open.builder.take().expect("one consuming page seal"),
                open.occupied,
            )
        };
        let values = builder
            .seal()
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        if occupied == 0 {
            Ok(None)
        } else {
            retain(&self.execution, Entry::Page(values), key, 0, self.control).map(Some)
        }
    }
}
