//! Diagnostic-only distinct physical node pages across retained roots.
use super::*;
use std::any::Any;

#[derive(Default)]
pub(crate) struct PageInspection {
    pages: Vec<Arc<dyn Any + Send + Sync>>,
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
            // Retain allocations while comparing identity so eviction cannot create a false share.
            let page = Arc::clone(directory) as Arc<dyn Any + Send + Sync>;
            if !inspection
                .pages
                .iter()
                .any(|retained| Arc::ptr_eq(retained, &page))
            {
                inspection.pages.push(page);
            }
        }
        Entry::Branch { left, right } => {
            visit(left, inspection);
            visit(right, inspection);
        }
    }
}
