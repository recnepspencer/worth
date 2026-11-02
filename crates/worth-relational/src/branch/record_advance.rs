//! Bounded authentic branch progression that did not author one specified record.
use super::{
    AdmittedRelationalBranchBasis, RelationalBranchReferenceObservation, RelationalBranchTarget,
    RelationalBranchVersion,
};
use crate::{
    history::data::{CanonicalCommitAuthorityKind, CommitId},
    runtime::RelationalRuntime,
    transactions::data::RecordRef,
};
use std::num::{NonZeroU64, NonZeroUsize};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, WorkCeilingDenial,
};
use worth_foundational::FoundationalBranchTarget;

#[derive(Clone, Copy, Debug)]
pub struct RelationalRecordAdvanceLimits {
    pub maximum_commits: NonZeroUsize,
    /// Examined authoritative record patches and authored intent slots.
    pub maximum_records: NonZeroUsize,
    /// Scratch backing and cold-frame acquisition ceiling. Resident envelopes
    /// are borrowed; their already retained payload is not copied or decoded.
    pub maximum_evidence_bytes: NonZeroU64,
}

#[derive(Debug)]
pub enum RelationalRecordAdvanceDenial {
    ForeignSource,
    SourceUnavailable,
    UnsupportedTransition,
    RecordChanged,
    BoundExceeded,
    Request(WorkCeilingDenial),
    Read(crate::execution::RelationalExecutionDenialCause),
}

/// Owner-issued observation of exactly these retained bases and record.
/// It proves neither permission nor a whole graph/domain projection.
pub struct RelationalRecordAdvanceObservation<'basis> {
    before: &'basis AdmittedRelationalBranchBasis,
    after: &'basis AdmittedRelationalBranchBasis,
    record: RecordRef,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RelationalRecordAdvanceObservation<'_> {
    pub fn corresponds_to(
        &self,
        before: &AdmittedRelationalBranchBasis,
        after: &AdmittedRelationalBranchBasis,
        record: &RecordRef,
    ) -> bool {
        self.before == before && self.after == after && &self.record == record
    }
}

struct Position {
    reference: RelationalBranchReferenceObservation,
    truth: RelationalBranchVersion,
}
use RelationalRecordAdvanceDenial as Denial;

impl RelationalRuntime {
    pub fn observe_untouched_record_advance<'basis>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        before: &'basis AdmittedRelationalBranchBasis,
        after: &'basis AdmittedRelationalBranchBasis,
        record: RecordRef,
        limits: RelationalRecordAdvanceLimits,
    ) -> Result<RelationalRecordAdvanceObservation<'basis>, Denial> {
        if before.identity().runtime_instance_id() != self.runtime_instance_id()
            || after.identity().runtime_instance_id() != self.runtime_instance_id()
            || before.identity() != after.identity()
        {
            return Err(Denial::ForeignSource);
        }
        self.compare_current_exact(after)
            .map_err(|_| Denial::SourceUnavailable)?;
        if before.descriptor().schema_commitment() != after.descriptor().schema_commitment() {
            return Err(Denial::UnsupportedTransition);
        }
        for basis in [before, after] {
            let FoundationalBranchTarget::Basis(target) = basis.reference().target() else {
                return Err(Denial::UnsupportedTransition);
            };
            if target.parent_commit_ids().len() > 1 {
                return Err(Denial::UnsupportedTransition);
            }
        }
        let bytes = size_of::<Position>()
            .checked_mul(8)
            .and_then(|base| {
                before
                    .identity()
                    .branch_id()
                    .0
                    .len()
                    .checked_mul(16)
                    .and_then(|names| base.checked_add(names))
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::BoundExceeded)?;
        if bytes > limits.maximum_evidence_bytes.get() {
            return Err(Denial::BoundExceeded);
        }
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        let before_frame = before
            .descriptor()
            .in_canonical_frame()
            .map_err(|_| Denial::UnsupportedTransition)?;
        let after_frame = after
            .descriptor()
            .in_canonical_frame()
            .map_err(|_| Denial::UnsupportedTransition)?;
        let mut answer = None;
        let outcome = execution.run_native_with_checkpoint(|context| {
            answer = Some(self.inspect_record_advance(
                execution,
                context,
                &before_frame,
                &after_frame,
                &record,
                limits,
            ));
        });
        match (outcome, answer) {
            (Ok(_), Some(Ok(()))) => Ok(RelationalRecordAdvanceObservation {
                before,
                after,
                record,
                _memory: memory,
            }),
            (Ok(_), Some(Err(cause))) => Err(cause),
            (
                Err(WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(Denial::Request(cause)),
            (Ok(_), None) => unreachable!("completed Native progression callback"),
        }
    }

    fn inspect_record_advance(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        before: &super::RelationalBranchBasisDescriptor,
        after: &super::RelationalBranchBasisDescriptor,
        record: &RecordRef,
        limits: RelationalRecordAdvanceLimits,
    ) -> Result<(), Denial> {
        let mut position = Position {
            reference: after.reference().clone(),
            truth: after.truth_version(),
        };
        let mut commits = 0;
        let mut records = 0;
        let expected_schema = match before.reference().target() {
            FoundationalBranchTarget::Basis(target) => *target.roots().schema_root(),
            _ => return Err(Denial::UnsupportedTransition),
        };
        while position.reference != *before.reference() || position.truth != before.truth_version()
        {
            checkpoint(context)?;
            if commits == limits.maximum_commits.get() {
                return Err(Denial::BoundExceeded);
            }
            let FoundationalBranchTarget::Basis(target) = position.reference.target() else {
                return Err(Denial::UnsupportedTransition);
            };
            let artifact = self
                .history
                .commit_artifact(CommitId(target.selected_commit_id()))
                .ok_or(Denial::SourceUnavailable)?;
            if artifact.parentage().as_slice().len() != 1
                || artifact.roots().schema_root() != &expected_schema
            {
                return Err(Denial::UnsupportedTransition);
            }
            let envelope = artifact
                .demand_envelope(execution, limits.maximum_evidence_bytes.get())
                .map_err(|cause| Denial::Read(cause.into()))?;
            let prior = envelope
                .branch_cell_checkpoint
                .as_ref()
                .ok_or(Denial::SourceUnavailable)?;
            if prior.runtime_instance_id != super::CANONICAL_RUNTIME_INSTANCE_ID
                || &prior.branch_id != before.branch_id()
                || envelope.branch_context != prior.branch_id
                || artifact.identity().authoring_branch() != &prior.branch_id
                || envelope.authority_kind != CanonicalCommitAuthorityKind::VersionedTransaction
                || envelope.schema_transition.is_some()
                || envelope.schema_continuation_descriptor.is_some()
                || envelope.schema_reconciliation_descriptor.is_some()
                || envelope.merge_execution_authority.is_some()
                || !envelope.merge_parent_branches.is_empty()
                || !envelope.merge_base_commits.is_empty()
            {
                return Err(Denial::UnsupportedTransition);
            }
            let FoundationalBranchTarget::Basis(prior_target) = prior.observation.target() else {
                return Err(Denial::UnsupportedTransition);
            };
            if artifact.parentage().as_slice()[0].0 != prior_target.selected_commit_id() {
                return Err(Denial::UnsupportedTransition);
            }
            // Use storage-owned roots, never provisional roots_for_commit.
            let post_target =
                FoundationalBranchTarget::basis(RelationalBranchTarget::from_commit_receipt(
                    super::CANONICAL_RUNTIME_INSTANCE_ID,
                    &artifact.receipt(),
                    artifact.roots().clone(),
                ));
            let post = super::relational_branch_observation(
                super::CANONICAL_RUNTIME_INSTANCE_ID,
                &prior.branch_id.0,
                post_target,
                prior
                    .observation
                    .generation()
                    .checked_advance()
                    .map_err(|_| Denial::UnsupportedTransition)?,
            )
            .map_err(|_| Denial::UnsupportedTransition)?;
            if post != position.reference
                || prior.truth_version.checked_advance() != Some(position.truth)
            {
                return Err(Denial::UnsupportedTransition);
            }
            let patches = &envelope.patch.authoritative_record_patches;
            if patches.len() > limits.maximum_records.get().saturating_sub(records) {
                return Err(Denial::BoundExceeded);
            }
            for patch in patches {
                checkpoint(context)?;
                records += 1;
                if could_author_record(&patch.target, record) {
                    return Err(Denial::RecordChanged);
                }
            }
            for target in envelope.authored_intent_targets() {
                checkpoint(context)?;
                if records == limits.maximum_records.get() {
                    return Err(Denial::BoundExceeded);
                }
                records += 1;
                if target
                    .as_ref()
                    .is_some_and(|target| could_author_record(target, record))
                {
                    return Err(Denial::RecordChanged);
                }
            }
            if let FoundationalBranchTarget::Basis(target) = prior.observation.target()
                && target.parent_commit_ids().len() > 1
            {
                return Err(Denial::UnsupportedTransition);
            }
            position = Position {
                reference: prior.observation.clone(),
                truth: prior.truth_version,
            };
            commits += 1;
        }
        Ok(())
    }
}

// Legacy wildcard-GEN intents cannot prove absence of authorship of a full-GEN
// record at the same slot, even if equal-valued writes leave no field patch.
fn could_author_record(target: &RecordRef, record: &RecordRef) -> bool {
    if target == record {
        return true;
    }
    match (target, record) {
        (RecordRef::Entity(target), RecordRef::Entity(record)) => {
            target.generation_value() == 0 && target.storage_id() == record.storage_id()
        }
        (RecordRef::Relation(target), RecordRef::Relation(record)) => {
            target.generation_value() == 0 && target.storage_id() == record.storage_id()
        }
        _ => false,
    }
}

fn checkpoint(context: &mut MapKernelContext<'_, '_>) -> Result<(), Denial> {
    (if context.has_accounting() {
        context.checkpoint(1)
    } else {
        context.poll()
    })
    .map_err(|cause| Denial::Request(WorkCeilingDenial::Stopped(cause)))
}
