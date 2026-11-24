//! A coupled part's naming plan travels with its commands. Every create
//! coordinate assembly rewrites in a part's batches (its scoped local keys, a
//! successor's chain keys and partitions, prefix-created endpoints) is
//! rewritten by the same closure in the part's plan, so each pending birth
//! still names the record its commands create. At completion the envelope's
//! plans compose into the one plan Relational's commit finalization validates
//! and mints, exactly as for a direct operation.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::symbols::data::client_key::ClientKey;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use contracts::lineage::data::{CommitNamingPlan, PendingBirthTarget, PendingMergeTarget};

/// Each pending create target the plan binds: births, split pieces and a
/// merge survivor created by this commit. Existing targets carry allocated
/// identities, which assembly never rewrites.
pub fn visit_targets_mut<E>(
    plan: &mut CommitNamingPlan,
    mut visit: impl FnMut(&mut PendingBirthTarget) -> Result<(), E>,
) -> Result<(), E> {
    for birth in &mut plan.pending_births {
        visit(&mut birth.target)?;
    }
    for split in &mut plan.pending_splits {
        for child in &mut split.children {
            visit(&mut child.target)?;
        }
    }
    for merge in &mut plan.pending_merges {
        if let PendingMergeTarget::Created(target) = &mut merge.target {
            visit(target)?;
        }
    }
    Ok(())
}

/// Each client key a target carries, in the order a create intent's own
/// visitor yields them: the created record's key, then created endpoints.
pub fn visit_client_keys_mut<E>(
    plan: &mut CommitNamingPlan,
    mut visit: impl FnMut(&mut ClientKey) -> Result<(), E>,
) -> Result<(), E> {
    visit_targets_mut(plan, |target| match target {
        PendingBirthTarget::CreatedEntity(created) => visit(&mut created.client_key),
        PendingBirthTarget::CreatedRelation(created) => {
            visit(&mut created.client_key)?;
            for endpoint in [&mut created.source, &mut created.target] {
                if let EntityReference::Created(endpoint) = endpoint {
                    visit(&mut endpoint.client_key)?;
                }
            }
            Ok(())
        }
    })
}

/// Each endpoint reference of a created relation target.
pub fn visit_endpoints_mut<E>(
    plan: &mut CommitNamingPlan,
    mut visit: impl FnMut(&mut EntityReference) -> Result<(), E>,
) -> Result<(), E> {
    visit_targets_mut(plan, |target| match target {
        PendingBirthTarget::CreatedEntity(_) => Ok(()),
        PendingBirthTarget::CreatedRelation(created) => {
            visit(&mut created.source)?;
            visit(&mut created.target)
        }
    })
}

/// The envelope's one naming plan: the authored prefix's own plan, then each
/// later authored stage's, then each producer chain's latest round. Nothing is
/// merged or deduplicated; Relational's finalization judges the whole plan,
/// so a name two parts mint refuses `DuplicateBirth` and a schemed record no
/// part names refuses `MissingBirth`, with no effect committed. Group
/// partitions compose the same way: claims over one scheme join one changed
/// scope, so a member two parts both group refuses `OverlappingMember`.
/// `None` when no member declared a name.
pub fn compose(
    execution: &ExecutionRequest<'_, '_>,
    root: Option<&CommitNamingPlan>,
    parts: impl IntoIterator<Item = Option<CommitNamingPlan>>,
) -> Result<Option<CommitNamingPlan>, TransactionCommitError> {
    let mut composed: Option<CommitNamingPlan> = root.cloned();
    execution
        .run_native_with_checkpoint(|context| {
            for part in parts.into_iter().flatten() {
                let charge = part.pending_births.len()
                    + part.pending_splits.len()
                    + part.pending_merges.len()
                    + part.postures.len()
                    + part.groups.len();
                checkpoint(context, charge as u64 + 1).map_err(custody::execution_stop)?;
                let plan = composed.get_or_insert_with(CommitNamingPlan::default);
                append(&mut plan.pending_births, part.pending_births)?;
                append(&mut plan.pending_splits, part.pending_splits)?;
                append(&mut plan.pending_merges, part.pending_merges)?;
                append(&mut plan.postures, part.postures)?;
                append(&mut plan.groups, part.groups)?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(composed)
}

fn append<T>(into: &mut Vec<T>, from: Vec<T>) -> Result<(), TransactionCommitError> {
    into.try_reserve_exact(from.len()).map_err(|_| {
        custody::execution_stop(
            contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
        )
    })?;
    into.extend(from);
    Ok(())
}
