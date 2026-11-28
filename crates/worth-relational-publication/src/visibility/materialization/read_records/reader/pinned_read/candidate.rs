//! Candidate-qualified native reads have no snapshot or Current observation door.
use super::*;
use contracts::execution::request_stage::checkpoint;
use contracts::indexes::data::applied_candidate_qualification::assemble_applied_candidate_read_qualification;
use mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;
pub mod changes;
use mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
pub(super) use changes::AppliedReadChanges;

/// The real applied proposal and assembly revision that supplied every row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalAppliedCandidateReadIdentity {
    proposal: RelationalMutationProposalIdentity,
    phase: CandidateReadPhase,
}

impl RelationalAppliedCandidateReadIdentity {
    pub const fn qualification(&self) -> RelationalAppliedCandidateReadQualification {
        assemble_applied_candidate_read_qualification(
            self.proposal.runtime_instance_id(),
            self.proposal.ordinal(),
            self.proposal.proposed_version_id(),
            self.phase,
        )
    }
    /// An assembly read includes its earlier revisions. The prepared image is
    /// issued only after the same proposal's complete assembly is admitted, so
    /// it includes that proposal's assembly prefixes too. Its candidate id is
    /// never interpreted as an assembly revision ordinal.
    pub fn includes_assembly_prefix(&self, prefix: &Self) -> bool {
        if !self.proposal.same_proposal_sequence(&prefix.proposal) {
            return false;
        }
        match (self.phase, prefix.phase) {
            (
                CandidateReadPhase::AssemblyRevision(at),
                CandidateReadPhase::AssemblyRevision(prior),
            ) => prior <= at,
            (CandidateReadPhase::PreparedCandidate(_), CandidateReadPhase::AssemblyRevision(_)) => {
                true
            }
            _ => false,
        }
    }
    pub const fn proposal(&self) -> &RelationalMutationProposalIdentity {
        &self.proposal
    }
}

/// The applied image is captured by the installed history owner using the
/// candidate's inherited, exact instrumentation and runtime coordinates.
pub struct AppliedCandidateCapture<'a> {
    pub history: &'a RelationalPreparationHistory,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub runtime_instance_id: u64,
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
    pub fn project_candidate(&self) -> worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView{
        self.project_read()
    }

    pub(crate) fn capture_applied<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AppliedCandidateCapture<'a>>,
        basis: &AdmittedRelationalBranchBasis,
        identity: &RelationalMutationProposalIdentity,
        revision: u64,
        state: &WorkingState,
        content: &worth_relational_branch::internal_port::content::publication::ContentPublication,
        symbols: &PendingSymbolMeanings,
        changed_records: &[contracts::transactions::data::primitives::RecordRef],
        aggregate: Option<mutation::aggregate_identity::candidate_read::AggregateCandidateRead>,
        previous: Option<&Self>,
    ) -> Result<Self, TransactionCommitError> {
        let inputs = inputs.into();
        let previous_root = if let Some(previous) = previous {
            let CandidateReadPhase::AssemblyRevision(prior_revision) = previous.snapshot.phase
            else {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "prepared read is not an assembly predecessor",
                    ),
                );
            };
            if !identity.same_proposal_sequence(&previous.snapshot.proposal)
                || prior_revision.checked_add(1) != Some(revision)
            {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "applied revision has a foreign or stale predecessor",
                    ),
                );
            }
            previous.resident_basis().basis().retain_root()
        } else {
            basis.owner_inner().root.clone()
        };
        let answer = execution
            .run_native_with_checkpoint(|context| {
                // The ordinary region owner hashes and captures the touched image.
                // Charge its slot traversal before entering that bounded operation.
                for id in state.partition_ids_iter() {
                    let slots = if state.clone_mode()
                        == worth_relational_storage::internal_port::storage::overlay::partition::PartitionCloneMode::GraphSparseEntities
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
                let mut dictionary = symbols.binding();
                if let Some(previous) = previous {
                    // The verified prior revision owns admission of its D root.
                    // Starting again from C loses that root's predecessor mark.
                    dictionary = symbols
                        .binding_from_applied_predecessor(previous.resident_basis().symbols())
                        .map_err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial)?;
                }
                let root = inputs
                    .history
                    .capture_applied_candidate_root(
                        execution,
                        &previous_root,
                        previous.map_or(basis.residency(), |previous| {
                            previous.resident_basis().basis().residency()
                        }),
                        state,
                        content,
                        &dictionary,
                    )
                    .map_err(|failure| {
                        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(format!(
                            "applied candidate root capture failed: {failure:?}"
                        ))
                    })?;
                Self::capture_root(
                    execution,
                    &dictionary,
                    inputs.instrumentation,
                    inputs.runtime_instance_id,
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
        dictionary: &RuntimeSymbolTable,
        instrumentation: &RuntimeInstrumentation,
        runtime_instance_id: u64,
        basis: &AdmittedRelationalBranchBasis,
        proposal: &RelationalMutationProposalIdentity,
        candidate_id: u64,
        root: Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
        changed_records: &[contracts::transactions::data::primitives::RecordRef],
    ) -> Result<Self, TransactionCommitError> {
        Self::capture_root(
            execution,
            dictionary,
            instrumentation,
            runtime_instance_id,
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
        dictionary: &RuntimeSymbolTable,
        instrumentation: &RuntimeInstrumentation,
        runtime_instance_id: u64,
        basis: &AdmittedRelationalBranchBasis,
        identity: RelationalAppliedCandidateReadIdentity,
        root: Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
        changed_records: &[contracts::transactions::data::primitives::RecordRef],
        aggregate: Option<mutation::aggregate_identity::candidate_read::AggregateCandidateRead>,
        previous_changes: Option<&AppliedReadChanges>,
    ) -> Result<Self, TransactionCommitError> {
        let schema = root.schema_authority();
        let evidence_basis = match identity.phase {
            CandidateReadPhase::AssemblyRevision(revision) => {
                contracts::query::data::planned_packets::QueryPlanEvidenceBasis::AppliedMutationCandidate {
                    proposal_ordinal: identity.proposal.ordinal(),
                    revision,
                }
            }
            CandidateReadPhase::PreparedCandidate(candidate_id) => {
                contracts::query::data::planned_packets::QueryPlanEvidenceBasis::PreparedMutationCandidate {
                    proposal_ordinal: identity.proposal.ordinal(),
                    candidate_id,
                }
            }
        };
        let context = QueryPlanContextId {
            runtime_instance_id,
            snapshot_id: contracts::snapshots::data::coordinates::SnapshotId(0),
            version_id: identity.proposal.proposed_version_id(),
            schema_version: schema.schema_version(),
            descriptor_semantics_version: schema.descriptor_semantics_version(),
            evidence_basis,
        };
        let fields = worth_relational_indexes::internal_port::indexes::authority::maintenance::applied_field::AppliedFieldImage::capture(
            execution,
            Arc::clone(&basis.owner_inner().root),
            Arc::clone(&root),
            identity.proposal.proposed_version_id(),
            changed_records,
            previous_changes.map(|changes| Arc::clone(&changes.fields)),
        )
        .map_err(request_stop)?;
        let source = worth_relational_branch::internal_port::history::retention::selected_snapshot::VisibilitySnapshotBasis::from_applied_root(
            basis.identity().branch_id().clone(),
            identity.proposal.proposed_version_id(),
            root,
        );
        let resident = ResidentVisibilitySnapshotBasis::admit(source, execution, dictionary)
            .map_err(request_stop)?;
        let applied_changes = changes::capture(
            execution,
            basis.owner_inner().root.id(),
            aggregate,
            previous_changes,
            fields,
        )?;
        Ok(Self {
            aggregate_naming: None,
            applied_changes: Some(applied_changes),
            current_version: identity.proposal.proposed_version_id(),
            instrumentation: instrumentation.clone(),
            snapshot: identity,
            basis: resident,
            context,
            indexes: BTreeMap::new(),
        })
    }
}

fn request_stop(
    cause: impl Into<contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}

#[doc(hidden)]
pub trait RelationalPinnedReadOwnerPort: Sized {
    fn capture_applied<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AppliedCandidateCapture<'a>>,
        basis: &AdmittedRelationalBranchBasis,
        identity: &RelationalMutationProposalIdentity,
        revision: u64,
        state: &WorkingState,
        content: &worth_relational_branch::internal_port::content::publication::ContentPublication,
        symbols: &PendingSymbolMeanings,
        changed_records: &[contracts::transactions::data::primitives::RecordRef],
        aggregate: Option<mutation::aggregate_identity::candidate_read::AggregateCandidateRead>,
        previous: Option<&Self>,
    ) -> Result<Self, TransactionCommitError>;
}

impl RelationalPinnedReadOwnerPort
    for RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>
{
    fn capture_applied<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AppliedCandidateCapture<'a>>,
        basis: &AdmittedRelationalBranchBasis,
        identity: &RelationalMutationProposalIdentity,
        revision: u64,
        state: &WorkingState,
        content: &worth_relational_branch::internal_port::content::publication::ContentPublication,
        symbols: &PendingSymbolMeanings,
        changed_records: &[contracts::transactions::data::primitives::RecordRef],
        aggregate: Option<mutation::aggregate_identity::candidate_read::AggregateCandidateRead>,
        previous: Option<&Self>,
    ) -> Result<Self, TransactionCommitError> {
        Self::capture_applied(
            execution,
            inputs,
            basis,
            identity,
            revision,
            state,
            content,
            symbols,
            changed_records,
            aggregate,
            previous,
        )
    }
}
