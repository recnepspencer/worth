//! Candidate-qualified native reads have no snapshot or Current observation door.
use super::*;
use crate::execution::request_stage::checkpoint;
mod changes;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    mvcc::RelationalMutationProposalIdentity,
    runtime::{PendingSymbolMeanings, RelationalPreparationRuntime, RuntimeServices},
    storage::overlay::{CurrentPartitionAccess, WorkingState},
    transactions::data::TransactionCommitError,
};
pub(super) use changes::AppliedReadChanges;

/// The real applied proposal and assembly revision that supplied every row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalAppliedCandidateReadIdentity {
    proposal: RelationalMutationProposalIdentity,
    phase: CandidateReadPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateReadPhase {
    AssemblyRevision(u64),
    PreparedCandidate(u64),
}

/// Compact descriptive qualification for values read from one applied image.
/// It retains no proposal, source context, record payload or runtime handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalAppliedCandidateReadQualification {
    runtime_instance_id: u64,
    proposal_ordinal: u64,
    proposed_version: crate::identity::data::VersionId,
    phase: CandidateReadPhase,
}
impl RelationalAppliedCandidateReadQualification {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn proposed_version_id(&self) -> crate::identity::data::VersionId {
        self.proposed_version
    }
}

impl RelationalAppliedCandidateReadIdentity {
    pub const fn qualification(&self) -> RelationalAppliedCandidateReadQualification {
        RelationalAppliedCandidateReadQualification {
            runtime_instance_id: self.proposal.runtime_instance_id(),
            proposal_ordinal: self.proposal.ordinal(),
            proposed_version: self.proposal.proposed_version_id(),
            phase: self.phase,
        }
    }
    pub const fn proposal(&self) -> &RelationalMutationProposalIdentity {
        &self.proposal
    }
}

impl RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
    pub(crate) fn applied_before_root(&self) -> u64 {
        self.applied_changes
            .as_ref()
            .expect("Native issued an applied read")
            .before_root
    }

    pub const fn candidate_identity(&self) -> &RelationalAppliedCandidateReadIdentity {
        &self.snapshot
    }

    /// Physical projections retain their candidate root; this cannot issue an
    /// operational snapshot or a branch observation.
    pub fn project_candidate(&self) -> super::super::super::projection::VisibilityProjectionView {
        super::super::super::projection::VisibilityProjectionView::from_pinned(self)
    }

    pub(crate) fn capture_applied(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        identity: &RelationalMutationProposalIdentity,
        revision: u64,
        state: &WorkingState,
        symbols: &PendingSymbolMeanings,
        changed_records: &[crate::transactions::data::RecordRef],
        aggregate: Option<crate::aggregate_identity::AggregateCandidateRead>,
        previous: Option<&Self>,
    ) -> Result<Self, TransactionCommitError> {
        let previous_root = if let Some(previous) = previous {
            let CandidateReadPhase::AssemblyRevision(prior_revision) = previous.snapshot.phase
            else {
                return Err(crate::mvcc::origin_denial(
                    "prepared read is not an assembly predecessor",
                ));
            };
            if !identity.same_proposal_sequence(&previous.snapshot.proposal)
                || prior_revision.checked_add(1) != Some(revision)
            {
                return Err(crate::mvcc::origin_denial(
                    "applied revision has a foreign or stale predecessor",
                ));
            }
            previous.resident_basis().root()
        } else {
            &basis.inner.root
        };
        let answer = execution
            .run_native_with_checkpoint(|context| {
                // The ordinary region owner hashes and captures the touched image.
                // Charge its slot traversal before entering that bounded operation.
                for id in state.partition_ids_iter() {
                    let slots = if state.clone_mode()
                        == crate::storage::overlay::PartitionCloneMode::GraphSparseEntities
                    {
                        state.mutation_journal().get(&id).map_or(0, |journal| {
                            journal
                                .entity_slots
                                .len()
                                .saturating_add(journal.relation_slots.len())
                                .saturating_add(journal.adjacency_slots.len())
                                .saturating_add(journal.reverse_adjacency_slots.len())
                        })
                    } else {
                        state.get_current_partition(id).map_or(0, |p| {
                            p.entity_arena()
                                .slot_count()
                                .saturating_add(p.relation_arena().slot_count())
                        })
                    };
                    checkpoint(context, slots.saturating_add(1) as u64).map_err(request_stop)?;
                }
                // Root capture admits the applied region's symbol closure into
                // this exact dictionary owner. Carry that same owner into the
                // pin; another detached binding loses the new admission marks
                // and a cold dictionary then refuses its otherwise valid read.
                let services = runtime.with_candidate_symbols(symbols).services;
                let root = runtime
                    .history
                    .capture_applied_candidate_root(
                        execution,
                        previous_root,
                        state,
                        &services.symbols,
                    )
                    .map_err(|failure| {
                        crate::mvcc::origin_denial(format!(
                            "applied candidate root capture failed: {failure:?}"
                        ))
                    })?;
                Self::capture_root(
                    execution,
                    &services,
                    basis,
                    RelationalAppliedCandidateReadIdentity {
                        proposal: identity.clone(),
                        phase: CandidateReadPhase::AssemblyRevision(revision),
                    },
                    root,
                    changed_records,
                    aggregate,
                    previous.and_then(|read| read.applied_changes.as_deref()),
                )
            })
            .map_err(request_stop)?;
        answer.0
    }
    pub(crate) fn capture_prepared(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        services: &RuntimeServices,
        basis: &AdmittedRelationalBranchBasis,
        proposal: &RelationalMutationProposalIdentity,
        candidate_id: u64,
        root: Arc<crate::branch::RelationalBranchRoot>,
        changed_records: &[crate::transactions::data::RecordRef],
    ) -> Result<Self, TransactionCommitError> {
        Self::capture_root(
            execution,
            services,
            basis,
            RelationalAppliedCandidateReadIdentity {
                proposal: proposal.clone(),
                phase: CandidateReadPhase::PreparedCandidate(candidate_id),
            },
            root,
            changed_records,
            None,
            None,
        )
    }

    fn capture_root(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        services: &RuntimeServices,
        basis: &AdmittedRelationalBranchBasis,
        identity: RelationalAppliedCandidateReadIdentity,
        root: Arc<crate::branch::RelationalBranchRoot>,
        changed_records: &[crate::transactions::data::RecordRef],
        aggregate: Option<crate::aggregate_identity::AggregateCandidateRead>,
        previous_changes: Option<&AppliedReadChanges>,
    ) -> Result<Self, TransactionCommitError> {
        let schema = root.schema_authority();
        let evidence_basis = match identity.phase {
            CandidateReadPhase::AssemblyRevision(revision) => {
                crate::query::data::QueryPlanEvidenceBasis::AppliedMutationCandidate {
                    proposal_ordinal: identity.proposal.ordinal(),
                    revision,
                }
            }
            CandidateReadPhase::PreparedCandidate(candidate_id) => {
                crate::query::data::QueryPlanEvidenceBasis::PreparedMutationCandidate {
                    proposal_ordinal: identity.proposal.ordinal(),
                    candidate_id,
                }
            }
        };
        let context = QueryPlanContextId {
            runtime_instance_id: services.runtime_instance_id(),
            snapshot_id: crate::snapshots::data::SnapshotId(0),
            version_id: identity.proposal.proposed_version_id(),
            schema_version: schema.schema_version(),
            descriptor_semantics_version: schema.descriptor_semantics_version(),
            evidence_basis,
        };
        let fields = crate::indexes::AppliedFieldImage::capture(
            execution,
            Arc::clone(&basis.inner.root),
            Arc::clone(&root),
            identity.proposal.proposed_version_id(),
            changed_records,
            previous_changes.map(|changes| Arc::clone(&changes.fields)),
        )
        .map_err(request_stop)?;
        let source = crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_applied_root(
            basis.identity().branch_id().clone(),
            identity.proposal.proposed_version_id(),
            root,
        );
        let resident = ResidentVisibilitySnapshotBasis::admit(source, execution, &services.symbols)
            .map_err(request_stop)?;
        let applied_changes = changes::capture(
            execution,
            basis.inner.root.id(),
            aggregate,
            previous_changes,
            fields,
        )?;
        Ok(Self {
            applied_changes: Some(applied_changes),
            current_version: identity.proposal.proposed_version_id(),
            instrumentation: services.instrumentation.clone(),
            snapshot: identity,
            basis: resident,
            context,
            indexes: BTreeMap::new(),
        })
    }
}

fn request_stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
