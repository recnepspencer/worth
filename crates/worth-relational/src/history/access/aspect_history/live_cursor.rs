//! Public finite cursor binding to the installed owner and original request.
use std::sync::Arc;

use super::{
    cursor::{prepare_page, RetainedCanonicalRecordHistory},
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryDenial as Denial,
    RelationalRecordHistoryPage, RelationalRecordHistoryPageLimits,
};
use crate::runtime::RelationalRuntime;

/// Position inside one genuine selected canonical commit. This does not prove
/// product ancestry; a composition owner supplies each selected component commit.
#[derive(Debug)]
pub struct RelationalCanonicalRecordHistoryCursor {
    pub(super) source: Arc<RetainedCanonicalRecordHistory>,
    pub(super) next_record: usize,
}

impl RelationalCanonicalRecordHistoryCursor {
    /// The new cursor source wrapper shares its canonical envelope and sealed selection.
    pub const fn owned_source_backing_bytes() -> usize {
        size_of::<RetainedCanonicalRecordHistory>() + 2 * size_of::<usize>()
    }

    /// Canonical record patches are the source-visit granule. Selected-aspect
    /// lookup is logarithmic in a record's operations; no patch payload is copied.
    /// All failures leave the accepted position unchanged.
    pub fn next_page(
        &mut self,
        runtime: &RelationalRuntime,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        limits: RelationalRecordHistoryPageLimits,
    ) -> Result<RelationalRecordHistoryPage, Denial> {
        let prepared = self.prepare_next_page(runtime, execution, limits)?;
        let (page, successor) = self.accept_page(prepared)?;
        *self = successor;
        Ok(page)
    }

    /// Stage finite native work before a composition owner completes fresh
    /// authorization/disclosure. No accepted cursor position changes here.
    pub fn prepare_next_page<'request, 'authority>(
        &self,
        runtime: &RelationalRuntime,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        limits: RelationalRecordHistoryPageLimits,
    ) -> Result<RelationalPreparedRecordHistoryPage<'request, 'authority>, Denial> {
        if self
            .source
            .retained
            .observation()
            .identity()
            .runtime_instance_id()
            != runtime.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        let page = super::request_stage::run(execution, |checkpoint| {
            prepare_page(
                &self.source,
                self.next_record,
                execution,
                checkpoint,
                limits,
            )
        })?;
        Ok(RelationalPreparedRecordHistoryPage {
            source: Arc::clone(&self.source),
            execution,
            expected_record: self.next_record,
            page,
        })
    }

    /// Return a sealed successor only for this cursor's still-current offset.
    /// The stored originating request controls the final handoff. The original
    /// cursor stays unchanged until its composition owner installs the successor.
    pub fn accept_page(
        &self,
        prepared: RelationalPreparedRecordHistoryPage<'_, '_>,
    ) -> Result<(RelationalRecordHistoryPage, Self), Denial> {
        if !Arc::ptr_eq(&self.source, &prepared.source)
            || self.next_record != prepared.expected_record
        {
            return Err(Denial::InvalidContinuation);
        }
        let (accepted, _) = prepared
            .execution
            .run_native(|_| {
                let successor = Self {
                    source: Arc::clone(&self.source),
                    next_record: prepared.page.next_record,
                };
                (prepared.page, successor)
            })
            .map_err(Denial::Request)?;
        Ok(accepted)
    }
}
