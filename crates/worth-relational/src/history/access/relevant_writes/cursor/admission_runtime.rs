//! Admit the exact installed custody needed by authored-write history.
use super::super::live_cursor::RelationalAuthoredWriteCursor;
use super::super::{
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial,
    RelationalAuthoredWriteSelector as Selector,
};
use super::admission::{admit_authored_write_history_cursor, Selection};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::commit_selection::RelationalSelectedCommit;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Select authored writes strictly after the genuine source root within the
    /// target's canonical single-parent prefix. Missing/merge accelerators refuse;
    /// equality-index predicates also require actual installed-definition coverage
    /// throughout the open source-to-target interval. No implicit rebuild occurs.
    pub fn authored_write_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        selectors: &[Selector],
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        self.admit_authored_write_history_cursor(
            execution,
            original,
            target,
            Selection::Ordinary(selectors),
        )
    }
    /// Complete conservative impact candidates for installed Source scopes.
    /// Shared-row scopes and disjoint fields on declared field kinds can appear.
    /// These pages
    /// prove neither exact query relevance nor correction cancellation.
    pub fn authored_managed_source_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        selectors: &[RelationalAuthoredManagedSourceSelector],
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        self.admit_authored_write_history_cursor(
            execution,
            original,
            target,
            Selection::Managed(selectors),
        )
    }
}
impl RelationalRuntime {
    fn admit_authored_write_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        input: Selection<'_>,
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        let (source, next) = admit_authored_write_history_cursor(
            &self.history,
            self.runtime_instance_id(),
            &self
                .schema_contract_runtime
                .custom_invariant_registries
                .source_history,
            execution,
            original,
            target,
            input,
            |_, commit| self.history().immutable_commit_receipt(commit).is_some(),
            |_, commit| self.history.authored_write_generation(commit),
            || {
                let catalog = self.indexes.catalog_binding().snapshot();
                move |definition: &super::super::RelationalAuthoredEntityIndexDefinition,
                      context: &mut worth_execution::MapKernelContext<'_, '_>| {
                    catalog.introduction(definition, context)
                }
            },
        )?;
        Ok(RelationalAuthoredWriteCursor { source, next })
    }
}
