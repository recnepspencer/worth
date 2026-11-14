//! An authenticated correction's root mapping does not depend on a cascade event.
use super::*;
use crate::storage::overlay::{CurrentPartitionAccess, PartitionAccess};
use crate::transactions::data::CreatedEntityRef;

impl RelationalRuntime {
    /// Retain correspondence for the actual corrective prefix. The caller has
    /// already authenticated its original correction meaning. This data grants
    /// no retirement/publication authority and selects no arbitrary draft subset.
    pub fn bind_authored_coupled_corrective_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        mut owners: RelationalAuthoredCoupledOwners,
        correspondence: (
            CommitId,
            (EntityId, CreatedEntityRef),
            Vec<(EntityId, CreatedEntityRef)>,
        ),
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        let (original_commit, anchor, co_anchors) = correspondence;
        let refuse = || origin_denial("corrective root correspondence differs from its original");
        let allocation_stop = |cause: worth_execution::ExecutionAllocationDenial| {
            crate::execution::denial::allocation_commit_error(TransactionCommitError::conflict(
                crate::mvcc::RelationalTransactionStagingDenial::from(cause).into_conflict(),
            ))
        };
        if installed.runtime_instance_id != self.runtime_instance_id()
            || owners.before.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || !owners.before.is_current()
            || owners.corrective_root.is_some()
            || matches!(owners.source, CoupledSource::Retirement { .. })
        {
            return Err(refuse());
        }
        let original = self
            .history_access()
            .demand_commit_envelope(execution, original_commit)
            .map_err(custody::execution_stop)?
            .ok_or_else(refuse)?;
        let count = co_anchors.len().checked_add(1).ok_or_else(refuse)?;
        let mut targets =
            worth_execution::ExecutionArrayBuilder::<(EntityId, CreatedEntityRef)>::allocate(
                count,
                execution.allocation_policy(),
            )
            .map_err(allocation_stop)?;
        for (old, target) in std::iter::once(anchor).chain(co_anchors) {
            execution
                .run_native_with_checkpoint(|context| {
                    for (known, created) in targets.elements() {
                        let work = created
                            .client_key
                            .owned_allocation_capacity_bytes()
                            .checked_add(target.client_key.owned_allocation_capacity_bytes())
                            .and_then(|bytes| bytes.checked_add(4))
                            .ok_or_else(refuse)?;
                        checkpoint(context, work).map_err(custody::execution_stop)?;
                        if *known == old || *created == target {
                            return Err(refuse());
                        }
                    }
                    Ok::<_, TransactionCommitError>(())
                })
                .map_err(custody::execution_stop)?
                .0?;
            owners
                .before
                .inner
                .root
                .demand_full_partition(execution, old.partition_id)
                .map_err(custody::execution_stop)?;
            if !owners
                .before
                .inner
                .root
                .get_current_partition(old.partition_id)
                .and_then(|partition| partition.entity_arena().get(&old))
                .is_some_and(|row| {
                    row.retired_at() == Some(original.commit.version_id)
                        && row.kind_id() == Some(target.kind_id)
                })
            {
                return Err(refuse());
            }
            targets.push((old, target)).map_err(allocation_stop)?;
        }
        let targets = targets.seal().map_err(allocation_stop)?;
        if let Some(plan) = owners.source.restoration() {
            let event = plan
                .retirement
                .authored_source_retirement()
                .ok_or_else(refuse)?;
            let canonical_count = event.co_anchors.len().checked_add(1).ok_or_else(refuse)?;
            execution
                .run_native_with_checkpoint(|context| {
                    if plan.retirement.commit.commit_id != original_commit
                        || canonical_count != count
                    {
                        return Err(refuse());
                    }
                    let canonical = plan.entities.get(..canonical_count).ok_or_else(refuse)?;
                    for (known, supplied) in canonical.iter().zip(targets.elements()) {
                        let work = known
                            .1
                            .client_key
                            .owned_allocation_capacity_bytes()
                            .checked_add(supplied.1.client_key.owned_allocation_capacity_bytes())
                            .and_then(|bytes| bytes.checked_add(4))
                            .ok_or_else(refuse)?;
                        checkpoint(context, work).map_err(custody::execution_stop)?;
                        if known != supplied {
                            return Err(refuse());
                        }
                    }
                    Ok::<_, TransactionCommitError>(())
                })
                .map_err(custody::execution_stop)?
                .0?;
        }
        owners.corrective_root = Some((original_commit, Arc::new(targets)));
        Ok(owners)
    }
}
