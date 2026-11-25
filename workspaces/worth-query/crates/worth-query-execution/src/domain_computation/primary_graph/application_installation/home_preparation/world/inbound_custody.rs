//! Actual World originals select the Native effects excluded from terminal readmission.
use super::*;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use QueryHomeCustodyReadmissionDenial as Denial;

pub(super) struct UnpublishedNativeCustody {
    pub(super) all: std::collections::BTreeSet<worth_relational::facade::history::CommitId>,
    pub(super) continuable: std::collections::BTreeMap<
        worth_relational::facade::history::CommitId,
        worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
    >,
}

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn unpublished_native_commits(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<UnpublishedNativeCustody, Denial> {
        let application = self.runtime.runtime();
        let owner = &application.inner.runtime.product_runtime_ref().owner_ref();
        let recovery = owner.recovery_port();
        let mut all = std::collections::BTreeSet::new();
        let mut continuable = std::collections::BTreeMap::new();
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
                    all.insert(receipt.commit_id);
                    let attempt = effects.attempt_identity();
                    if let Some(durable_owner) = attempt.owner_identity().durable() {
                        continuable.insert(
                            receipt.commit_id,
                            worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
                                durable_owner,
                                attempt.ordinal(),
                            ),
                        );
                    }
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
                    all.insert(receipt.commit_id);
                }
            }
            after = page.next_after().cloned();
            if after.is_none() {
                return Ok(UnpublishedNativeCustody { all, continuable });
            }
        }
    }
}
