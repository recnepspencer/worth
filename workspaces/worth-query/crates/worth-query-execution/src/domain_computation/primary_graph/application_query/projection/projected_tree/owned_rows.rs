//! One immutable row owner carries deep disposal backing and its original payer.
use super::*;
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Default)]
pub struct OwnedProjectionRows {
    rows: Vec<WorthQueryApplicationProjectionNode>,
    managed: Option<ManagedProjectionStorage>,
}
struct ManagedProjectionStorage {
    disposal: ProjectionDisposal,
    payload_bytes: usize,
    _allocation: Arc<SourceFactsAllocation>,
}
pub struct ProjectionDisposal {
    stack: Vec<DropFrame>,
}
struct DropFrame {
    relations: std::vec::IntoIter<WorthQueryApplicationProjectedRelation>,
    rows: std::vec::IntoIter<WorthQueryApplicationProjectionNode>,
}
impl OwnedProjectionRows {
    pub(in crate::domain_computation::primary_graph::application_query) fn ordinary(
        rows: Vec<WorthQueryApplicationProjectionNode>,
    ) -> Self {
        Self {
            rows,
            managed: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn managed(
        rows: Vec<WorthQueryApplicationProjectionNode>,
        disposal: ProjectionDisposal,
        payload_bytes: usize,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Self {
        Self {
            rows,
            managed: Some(ManagedProjectionStorage {
                disposal,
                payload_bytes,
                _allocation: allocation,
            }),
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn payload_bytes(
        &self,
    ) -> usize {
        match &self.managed {
            Some(storage) => storage.payload_bytes,
            None => self.rows.iter().fold(
                self.rows
                    .capacity()
                    .saturating_mul(size_of::<WorthQueryApplicationProjectionNode>()),
                |bytes, row| bytes.saturating_add(row.retained_bytes()),
            ),
        }
    }
}
impl std::ops::Deref for OwnedProjectionRows {
    type Target = Vec<WorthQueryApplicationProjectionNode>;
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}
impl Drop for OwnedProjectionRows {
    fn drop(&mut self) {
        if let Some(storage) = &mut self.managed {
            storage.disposal.clear_rows(&mut self.rows);
        }
        // Fields drop in order: empty rows backing, disposal backing, payer.
    }
}
impl ProjectionDisposal {
    pub(in crate::domain_computation::primary_graph::application_query) fn admitted(
        execution: &ExecutionRequest<'_, '_>,
        allocation: &SourceFactsAllocation,
        maximum_depth: usize,
    ) -> Result<Self, Cause> {
        if maximum_depth == 0 {
            return Err(Cause::ChargedBytesOverflow);
        }
        allocation.claim_slice::<DropFrame>(execution, maximum_depth)?;
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(maximum_depth)
            .map_err(|_| Cause::AllocationFailed)?;
        Ok(Self { stack })
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn clear_rows(
        &mut self,
        rows: &mut Vec<WorthQueryApplicationProjectionNode>,
    ) {
        while let Some(mut row) = rows.pop() {
            self.clear_node(&mut row);
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn clear_node(
        &mut self,
        node: &mut WorthQueryApplicationProjectionNode,
    ) {
        self.descend(std::mem::take(&mut node.relations));
        while let Some(frame) = self.stack.last_mut() {
            if let Some(mut child) = frame.rows.next() {
                let relations = std::mem::take(&mut child.relations);
                drop(child); // all nested ownership has already moved to the continuation
                self.descend(relations);
            } else if let Some(mut relation) = frame.relations.next() {
                frame.rows = std::mem::take(&mut relation.rows).into_iter();
            } else {
                self.stack.pop();
            }
        }
    }
    fn descend(&mut self, relations: Vec<WorthQueryApplicationProjectedRelation>) {
        if relations.is_empty() {
            return;
        }
        assert!(
            self.stack.len() < self.stack.capacity(),
            "producer preserves its compiled disposal depth"
        );
        self.stack.push(DropFrame {
            relations: relations.into_iter(),
            rows: Vec::new().into_iter(),
        });
    }
}
