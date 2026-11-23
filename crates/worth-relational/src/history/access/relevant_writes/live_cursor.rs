use super::{
    cursor::RetainedAuthoredWriteSelection,
    page::{
        RelationalAuthoredWritePage, RelationalAuthoredWritePageLimits,
        RelationalPreparedAuthoredWritePage,
    },
    RelationalAuthoredWriteDenial as Denial,
};
use crate::runtime::RelationalRuntime;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

/// A finite continuation over one owner-selected canonical single-parent prefix.
#[derive(Debug)]
pub struct RelationalAuthoredWriteCursor {
    pub(super) source: Arc<RetainedAuthoredWriteSelection>,
    pub(super) next: u64,
}
impl RelationalAuthoredWriteCursor {
    /// Each page probes the selected keys in the key tree and their per-key
    /// postings, then resolves only emitted canonical occurrences. Prefix proof
    /// uses its own logarithmic lookup. No unrelated records or sibling branches
    /// are enumerated. Refusal changes no position.
    pub fn prepare_next_page<'request, 'authority>(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'request, 'authority>,
        limits: RelationalAuthoredWritePageLimits,
    ) -> Result<RelationalPreparedAuthoredWritePage<'request, 'authority>, Denial> {
        let runtime_instance_id = runtime.runtime_instance_id();
        self.source.require_runtime(runtime_instance_id)?;
        let page = super::run(execution, |context| {
            self.source
                .prepare_page(self.next, execution, context, limits, |commit| {
                    runtime
                        .history_access()
                        .demand_commit_envelope(&execution, commit)
                        .map_err(Denial::Execution)?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)
                })
        })?;
        Ok(RelationalPreparedAuthoredWritePage {
            source: Arc::clone(&self.source),
            expected: self.next,
            page,
            execution,
        })
    }

    /// Final handoff polls the same original request retained by the preparation.
    pub fn accept_page(
        &self,
        prepared: RelationalPreparedAuthoredWritePage<'_, '_>,
    ) -> Result<(RelationalAuthoredWritePage, Self), Denial> {
        if !Arc::ptr_eq(&self.source, &prepared.source) || self.next != prepared.expected {
            return Err(Denial::InvalidContinuation);
        }
        prepared.execution.consult().map_err(Denial::Request)?;
        let successor = Self {
            source: Arc::clone(&self.source),
            next: prepared.page.next,
        };
        Ok((prepared.page, successor))
    }

    pub fn next_page(
        &mut self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
        limits: RelationalAuthoredWritePageLimits,
    ) -> Result<RelationalAuthoredWritePage, Denial> {
        let prepared = self.prepare_next_page(runtime, execution, limits)?;
        let (page, successor) = self.accept_page(prepared)?;
        *self = successor;
        Ok(page)
    }
}
