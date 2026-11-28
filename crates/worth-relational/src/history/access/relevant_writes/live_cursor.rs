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
use worth_relational_branch::internal_port::history::access::relevant_writes::page::assemble_prepared_authored_write_page;
use worth_relational_branch::internal_port::history::access::relevant_writes::page::RelationalAuthoredWritePageFieldsPort as _;
use worth_relational_branch::internal_port::history::access::relevant_writes::page::RelationalPreparedAuthoredWritePageFieldsPort as _;

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
                        .history()
                        .demand_commit_envelope(&execution, commit)
                        .map_err(Denial::Execution)?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)
                })
        })?;
        Ok(assemble_prepared_authored_write_page(
            Arc::clone(&self.source),
            self.next,
            page,
            execution,
        ))
    }

    /// Final handoff polls the same original request retained by the preparation.
    pub fn accept_page(
        &self,
        prepared: RelationalPreparedAuthoredWritePage<'_, '_>,
    ) -> Result<(RelationalAuthoredWritePage, Self), Denial> {
        if !Arc::ptr_eq(&self.source, prepared.owner_source())
            || self.next != (*prepared.owner_expected())
        {
            return Err(Denial::InvalidContinuation);
        }
        (*prepared.owner_execution())
            .consult()
            .map_err(Denial::Request)?;
        let successor = Self {
            source: Arc::clone(&self.source),
            next: (*prepared.page().owner_next()),
        };
        Ok((prepared.into_page(), successor))
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
