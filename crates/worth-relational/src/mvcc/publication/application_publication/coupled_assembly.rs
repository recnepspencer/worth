//! Query supplies producer execution; Native owns the applied image and writer roster.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::authored_restoration::RelationalAuthoredRestorationFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_owners::RelationalAuthoredCoupledOwnersOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthorityFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_owners::RelationalAuthoredCoupledOwnersFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::custody::RelationalRequiredOutputPublicationFieldsPort as _;
use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::candidate::RelationalPinnedReadOwnerPort as _;
use self::retained_assembly::{
    BoundAuthoredCoupling, CoupledAuthoredStagePart, CoupledPublicationPart,
};
use super::custody;
use super::*;
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;
use crate::authority::commit::phases::prepare::PreparedWorkingStateScope;
use crate::execution::request_stage::checkpoint;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::mvcc::BranchBoundRelationalTransaction;
use crate::runtime::RelationalPreparationRuntime;
use crate::visibility::materialization::read_records::reader::pinned_read::candidate::RelationalAppliedCandidateReadIdentity;
use crate::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_mutation::internal_port::mvcc::transaction::intent::RelationalTransactionIntentOwnerPort as _;
pub(crate) use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::{assembly_failure, authored_writes, chain, completion, contribution, corruption_commands, disjoint_field_patches, fresh_references, inputs, local_keys, net_plan, parent_read_footprint, part_naming, participant_admission, producer_posture, refusals, retained_assembly, roster_completion, route_selection, source_effects, suffix_scope, touch_projection, writer_custody};
mod input_issuance;
pub use inputs::RelationalRequiredOutputInput;
mod aggregate_read;
mod candidate_completion;
mod candidate_names;
mod continuation;
mod managed_source;
mod managed_source_alignment;
pub use continuation::RelationalCoupledProducerContinuation;
pub use managed_source_alignment::ManagedIndexParentReadFootprint;
mod proposal_contribution;
pub use contribution::RelationalCoupledContribution;
mod ordinary_roles;
mod part;
mod part_corruption;
mod prepared_route;
mod stage;
pub use producer_posture::RelationalCoupledProducerPosture;
mod scoped_preparation;
mod succession;
mod touches;
pub(crate) use completion::CoupledAppliedCandidate;
pub use completion::RelationalCoupledAuthoredPublication;
pub use touches::{
    RelationalCoupledAppendTouches, RelationalCoupledChainOverlay,
    RelationalCoupledOverlayEndpoints, RelationalCoupledOverlayRelation,
};

/// Installed producer dispatch over the real unpublished authored candidate.
/// It may append only the Native-admitted owners' own successor publications.
pub trait RelationalAuthoredCoupledProducerAssembly {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<std::ops::ControlFlow<()>, RelationalCoupledAssemblyFailure>;
}

/// A transaction whose authored anchors and exact dependent owner set were
/// admitted by Native. It can only enter scoped coupled preparation or be dropped.
#[derive(Debug)]
pub struct RelationalAuthoredCoupledTransaction {
    pub(crate) transaction: BranchBoundRelationalTransaction,
}

/// An applied proposal phase, neither a validated proposal nor a publication.
/// Only Native constructs this borrowed assembler and its candidate read pins.
pub struct RelationalAuthoredCoupledCandidate<'a> {
    pub(crate) runtime: &'a RelationalPreparationRuntime,
    pub(crate) transaction: &'a mut BranchBoundRelationalTransaction,
    pub(crate) prepared: &'a mut PreparedWorkingStateScope,
    pub(crate) mutation: &'a mut AppliedMutationCore,
    pub(crate) identity: &'a RelationalMutationProposalIdentity,
    pub(crate) binding: &'a BoundAuthoredCoupling,
    input_issuance: std::cell::RefCell<Option<Arc<inputs::InputIssuance>>>,
    pub(crate) parts: Vec<CoupledPublicationPart>,
    pub(crate) stages: Vec<CoupledAuthoredStagePart>,
    /// Exact applied prefix; later authored stages retain their own spans.
    authored_prefix: std::ops::Range<usize>,
    continuation: continuation::ContinuationState,
    fresh_records: net_plan::CreatorCustody,
    authored_writes: authored_writes::AuthoredWrites,
    read: std::cell::RefCell<Option<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>>>,
    /// The envelope's names over `read`'s applied state (#426).
    candidate_names: std::cell::RefCell<candidate_names::CandidateNameCache>,
    /// Successor rounds applied to earlier parts' chains; each is still one
    /// applied part in the pin, local-key and continuation sequence.
    successor_rounds: usize,
    /// Every live member's writes and references (#119 guards).
    indexes: succession::indexes::MemberIndexes,
    /// The authored prefix's entries in the accumulators, and its content.
    prefix_span: succession::segment::MemberSpan,
    prefix_content: crate::content::RelationalContentAttachments,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Every applied part, authored stage or producer, in apply order. Pins,
    /// local keys and successor reads are numbered over this one sequence.
    pub(crate) fn applied_parts(&self) -> usize {
        self.parts.len() + self.stages.len() + self.successor_rounds
    }

    /// The full-generation source admitted by Native for this authored prefix.
    /// Descriptive identity only; reading D still requires its applied read pin.
    pub fn authored_source(&self) -> crate::identity::data::EntityId {
        self.binding.owners.source().subject()
    }

    /// Classify a parent read through the canonical roster. An absent entry
    /// grants no writer: Query may only verify an already current output, or
    /// ask `require_unlisted_writer` before preparing a new participant.
    pub fn read_parent_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError> {
        self.parent_owner(execution, family, source, partition, false)
    }

    /// The parent owner a successor round of the route's existing part reads
    /// (§1.5): the same canonical entry, which that part already claimed, so
    /// the route is expected to be seen. Any other route reads `read_parent_owner`.
    pub fn read_successor_parent_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError> {
        if self
            .prepared_route_index(execution, family, source, partition)?
            .is_none()
        {
            return self.read_parent_owner(execution, family, source, partition);
        }
        self.parent_owner(execution, family, source, partition, true)
    }

    fn parent_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
        successor: bool,
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError> {
        let original = self.catalog_source(execution, source)?;
        let owner = self
            .binding
            .owners
            .recorded_owner(execution, family, original, partition)?;
        let Some(owner) = owner else {
            return Ok(None);
        };
        if matches!(owner, RelationalDeclaredCoupledOwner::Retiring)
            && original != source
            && self.binding.owners.source().restoration().is_some()
        {
            return self
                .restored_output_input(execution, family, original, source, partition)
                .map(|input| Some(RelationalDeclaredCoupledOwner::Restored(input)));
        }
        if let RelationalDeclaredCoupledOwner::Surviving(previous)
        | RelationalDeclaredCoupledOwner::RequiredSurviving(previous) = &owner
            && !successor
        {
            execution
                .run_native_with_checkpoint(|context| {
                    participant_admission::require_unseen_route(
                        context,
                        &self.parts,
                        &previous.origin().meaning,
                    )
                })
                .map_err(custody::execution_stop)?
                .0?;
        }
        Ok(Some(owner))
    }

    fn catalog_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: crate::identity::data::EntityId,
    ) -> Result<crate::identity::data::EntityId, TransactionCommitError> {
        execution.run_native_with_checkpoint(|context| {
            if let Some(restoration) = self.binding.owners.source().restoration() {
                for (old, created) in restoration.entities() {
                    checkpoint(context, self.mutation.outcome.created_entities.resolve_work_bound(created).ok_or_else(|| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(custody::execution_stop)?;
                    if self.mutation.outcome.created_entities.resolve(created) == Some(source) {
                        return Ok::<_, TransactionCommitError>(*old);
                    }
                }
            }
            Ok(source)
        }).map_err(custody::execution_stop)?.0
    }

    /// No absent catalog route may become a fresh retirement/restoration writer.
    /// Ordinary Edit source admission is still decided by its current owner.
    pub fn require_unlisted_writer(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<(), TransactionCommitError> {
        if self.binding.owners.source().is_edit() {
            return Ok(());
        }
        let original = self.catalog_source(execution, source)?;
        Err(part::reject_extra_owner(&self.binding.owners, source, format!(
            "site=catalog-route-absent; family={family}; requested_source={source:?}; original_source={original:?}; partition={partition:?}",
        )))
    }

    pub fn pin_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>, TransactionCommitError>
    {
        execution
            .run_native_with_checkpoint(|context| {
                checkpoint(context, 1).map_err(custody::execution_stop)
            })
            .map_err(custody::execution_stop)?
            .0?;
        if let Some(read) = self.read.borrow().as_ref() {
            return Ok(read.clone());
        }
        let read = self.capture_pin(execution, self.aggregate_read(execution)?)?;
        self.read.replace(Some(read.clone()));
        Ok(read)
    }

    /// The retracted image a successor round applies over (#119 §2.3). It
    /// is never cached and has no aggregate projection: another chain's
    /// records may name one the round has yet to re-create, so the derived
    /// laws cannot be read there. The pin after the round projects them.
    pub(super) fn retracted_pin(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>, TransactionCommitError>
    {
        debug_assert!(self.read.borrow().is_none(), "a retracted candidate");
        self.capture_pin(execution, None)
    }

    fn capture_pin(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        aggregate: Option<crate::aggregate_identity::AggregateCandidateRead>,
    ) -> Result<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>, TransactionCommitError>
    {
        RelationalPinnedRead::capture_applied(
            execution,
            self.runtime,
            self.binding.owners.before(),
            self.identity,
            self.applied_parts() as u64,
            self.mutation.state(),
            &self.mutation.content,
            self.mutation.candidate_symbols(),
            &self.mutation.outcome.effect.publication.changed_records,
            aggregate,
            None,
        )
    }

    /// Whether the unpublished envelope claims group scheme `scheme` (#361).
    pub fn claims_group_scheme(&self, scheme: &str) -> bool {
        let stages = self.stages.iter().map(|stage| stage.naming.as_ref());
        let parts = self.parts.iter().map(|part| part.naming.as_ref());
        let root = self.transaction.naming_plan.as_deref();
        part_naming::claims_group_scheme(root, stages.chain(parts), scheme)
    }
}

impl BranchBoundRelationalTransaction {
    /// Attach installed producer dispatch to a Native-derived exact owner set.
    /// The binding is retained across abandoned preparation rounds, never history.
    pub fn bind_authored_coupled_producers(
        mut self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        owners: RelationalAuthoredCoupledOwners,
        retirement_admission: Option<worth_proof::AdmittedAuthoredOutputRetirement>,
    ) -> Result<RelationalAuthoredCoupledTransaction, TransactionCommitError> {
        if self.authored_coupling.is_some()
            || installed.runtime_instance_id() != self.basis.identity().runtime_instance_id()
            || owners.before() != &self.basis
            || self.intent.materialization_mode().is_some()
            || !self.merge_parent_bases.is_empty()
            || retirement_admission
                .as_ref()
                .map_or(!owners.retirements().is_empty(), |admission| {
                    admission.publication_count() != owners.retirements().len()
                        || owners.retirements().iter().any(|prior| {
                            !installed
                                .authority()
                                .admits_authored_output_retirement(admission, prior.authority())
                        })
                })
        {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                owners.source().subject(),
            ));
        }
        self.authored_coupling = Some(Arc::new(BoundAuthoredCoupling {
            owners,
            applied: std::sync::Mutex::new(None),
            completed: std::sync::Mutex::new(None),
            installed: installed.clone(),
            retirement_admission: retirement_admission.map(Arc::new),
        }));
        Ok(RelationalAuthoredCoupledTransaction { transaction: self })
    }
}
