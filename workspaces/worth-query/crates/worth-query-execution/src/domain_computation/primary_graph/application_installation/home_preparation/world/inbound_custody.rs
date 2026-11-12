//! Actual World originals select the Native effects excluded from terminal readmission.
use super::*;
use QueryHomeCustodyReadmissionDenial as Denial;

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn unpublished_native_commits(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<std::collections::BTreeSet<worth_relational::facade::history::CommitId>, Denial>
    {
        let application = self.runtime.runtime();
        let owner = &application.runtime.product_runtime.owner;
        let recovery = owner.recovery_port();
        let mut commits = std::collections::BTreeSet::new();
        let mut after = None;
        loop {
            execution.consult().map_err(Denial::Execution)?;
            let page = owner
                .inspection_port()
                .recovery_page(after.as_ref(), std::num::NonZeroUsize::new(32).unwrap())
                .map_err(Denial::World)?;
            for row in page.rows() {
                let effects = recovery
                    .inspect_recovered_effects(row.handle())
                    .map_err(Denial::Recovery)?;
                if let Some(receipt) = effects.native_receipt() {
                    commits.insert(receipt.commit_id);
                }
            }
            after = page.next_after().cloned();
            if after.is_none() {
                break;
            }
        }
        let mut after = None;
        loop {
            let page = recovery
                .released_original_page(
                    execution,
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).unwrap(),
                )
                .map_err(Denial::ReleasedSource)?;
            for original in page.rows() {
                if let Some(receipt) = original.native_receipt() {
                    commits.insert(receipt.commit_id);
                }
            }
            after = page.next_after().cloned();
            if after.is_none() {
                return Ok(commits);
            }
        }
    }
}
