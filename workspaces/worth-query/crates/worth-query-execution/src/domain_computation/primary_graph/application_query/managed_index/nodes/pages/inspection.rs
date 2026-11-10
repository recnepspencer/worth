//! Diagnostic-only distinct physical node pages across retained roots.
use super::*;
use std::{any::Any, collections::BTreeMap};

#[derive(Default)]
pub(crate) struct PageInspection {
    pages: BTreeMap<usize, Arc<dyn Any + Send + Sync>>,
    pub(in super::super::super::super) references: usize,
}
impl PageInspection {
    pub(in super::super::super::super) fn distinct(&self) -> usize {
        self.pages.len()
    }
}
impl<Node: Send + Sync + 'static> NodePages<Node> {
    pub(in super::super::super::super) fn inspect_pages(&self, inspection: &mut PageInspection) {
        if let Some(root) = &self.root {
            visit(root, inspection);
        }
    }
}
fn visit<Node: Send + Sync + 'static>(
    directory: &Arc<Directory<Node>>,
    inspection: &mut PageInspection,
) {
    match &directory.entry {
        Entry::Page(_) => {
            inspection.references += 1;
            // Keep each inspected allocation alive until counting finishes, so a
            // concurrent eviction cannot recycle its address into a false share.
            inspection
                .pages
                .entry(Arc::as_ptr(directory) as usize)
                .or_insert_with(|| Arc::clone(directory) as Arc<dyn Any + Send + Sync>);
        }
        Entry::Branch { left, right } => {
            visit(left, inspection);
            visit(right, inspection);
        }
    }
}
