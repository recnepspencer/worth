//! One ordered union of touched paths; virtual growth is never installed twice.
use super::*;
use std::iter::Peekable;

pub(super) struct Batch<'control, 'call, 'request, I: Iterator> {
    pub(super) pages: Peekable<I>,
    pub(super) execution: ExecutionRequest<'request, 'request>,
    pub(super) control: &'control WorthQueryManagedIndexControl<'call>,
    #[cfg(test)]
    pub(super) allocations: &'control Cell<usize>,
}
impl<'pages, 'request: 'pages, Node: Copy + 'pages, I> Batch<'_, '_, 'request, I>
where
    I: Iterator<Item = (&'pages usize, &'pages Rc<WorkingPage<'request, Node>>)>,
{
    pub(super) fn subtree(
        &mut self,
        previous: NodePages<Node>,
        start: usize,
        height: u32,
    ) -> Result<Option<Arc<Directory<Node>>>, Stop> {
        self.control.checkpoint(1)?;
        let touched = self
            .pages
            .peek()
            .is_some_and(|(key, _)| **key >> height == start >> height);
        if !touched {
            return self.unchanged(previous, height);
        }
        if height == 0 {
            let (_, page) = self.pages.next().expect("peeked current touched page");
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
            return if occupied == 0 {
                Ok(None)
            } else {
                retain(&self.execution, Entry::Page(values), self.control).map(Some)
            };
        }
        let (left, right) = if previous.height < height {
            // The actual old root is the left subtree of these virtual levels.
            (previous, NodePages::default())
        } else {
            let (left, right) = match previous.root.as_deref().map(|old| &old.entry) {
                Some(Entry::Branch { left, right }) => (left.clone(), right.clone()),
                Some(Entry::Page(_)) => unreachable!("private branch directory height"),
                None => (None, None),
            };
            (
                NodePages {
                    root: left,
                    height: height - 1,
                },
                NodePages {
                    root: right,
                    height: height - 1,
                },
            )
        };
        let left = self.subtree(left, start, height - 1)?;
        let right = self.subtree(right, start + (1 << (height - 1)), height - 1)?;
        self.branch(left, right)
    }
    fn unchanged(
        &self,
        previous: NodePages<Node>,
        height: u32,
    ) -> Result<Option<Arc<Directory<Node>>>, Stop> {
        let mut root = previous.root;
        if root.is_some() {
            for _ in previous.height..height {
                root = self.branch(root, None)?;
            }
        }
        Ok(root)
    }
    fn branch(
        &self,
        left: Option<Arc<Directory<Node>>>,
        right: Option<Arc<Directory<Node>>>,
    ) -> Result<Option<Arc<Directory<Node>>>, Stop> {
        let result = branch(&self.execution, left, right, self.control)?;
        #[cfg(test)]
        if result.is_some() {
            self.allocations.set(self.allocations.get() + 1);
        }
        Ok(result)
    }
}
