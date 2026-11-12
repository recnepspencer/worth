//! Query supplies producer execution; Native owns the applied image and writer roster.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::{
    authority::commit::phases::{
        applied_mutation::AppliedMutationCore, prepare::PreparedWorkingStateScope,
    },
    mvcc::{BranchBoundRelationalTransaction, RelationalMutationProposalIdentity},
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalPreparationRuntime,
    },
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
mod adoption;
mod aggregate_read;
mod completion;
mod contribution;
mod fresh_references;
mod local_keys;
mod part;
mod part_apply;
mod part_corruption;
mod scoped_preparation;
mod writer_custody;
pub(crate) use completion::CoupledAppliedCandidate;
pub use completion::RelationalCoupledAuthoredPublication;

/// Installed producer dispatch over the real unpublished authored candidate.
/// It may append only the Native-admitted owners' own successor publications.
pub trait RelationalAuthoredCoupledProducerAssembly {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<std::ops::ControlFlow<()>, RelationalCoupledAssemblyFailure>;
}

/// Retains Native cold-symbol discovery so the ordinary round can retry it.
#[derive(Debug)]
pub struct RelationalCoupledAssemblyFailure {
    pub(crate) cause: crate::authority::mutation::MutationApplyDenial,
}
impl From<TransactionCommitError> for RelationalCoupledAssemblyFailure {
    fn from(error: TransactionCommitError) -> Self {
        Self {
            cause: error.into(),
        }
    }
}
impl RelationalCoupledAssemblyFailure {
    pub(crate) fn from_apply(cause: crate::authority::mutation::MutationApplyDenial) -> Self {
        Self { cause }
    }
}

pub(crate) struct BoundAuthoredCoupling {
    pub(crate) owners: RelationalAuthoredCoupledOwners,
    pub(crate) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(crate) applied: std::sync::Mutex<Option<CoupledAppliedCandidate>>,
    pub(crate) completed: std::sync::Mutex<Option<RequiredOutputPublicationAdmission>>,
    pub(crate) retirement_admission: Option<Arc<worth_proof::AdmittedAuthoredOutputRetirement>>,
}
impl std::fmt::Debug for BoundAuthoredCoupling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundAuthoredCoupling")
            .field("owners", &self.owners)
            .finish_non_exhaustive()
    }
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
    pub(crate) parts: Vec<CoupledPublicationPart>,
}

pub(crate) struct CoupledPublicationPart {
    pub(crate) publication: RequiredOutputPublication,
    pub(crate) commands: crate::mvcc::OwnedNormalizedPlan,
    pub(crate) records: Vec<crate::transactions::data::RecordRef>,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Classify an actual candidate source through the same canonical roster
    /// that admitted this assembly. The caller cannot submit an owner set.
    pub fn declared_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<RelationalDeclaredCoupledOwner<'_>, TransactionCommitError> {
        let original = execution.run_native_with_checkpoint(|context| {
            if let Some(restoration) = &self.binding.owners.restoration {
                for (old, created) in &restoration.entities {
                    checkpoint(context, self.mutation.outcome.created_entities.resolve_work_bound(created).ok_or_else(|| custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow))?).map_err(custody::execution_stop)?;
                    if self.mutation.outcome.created_entities.resolve(created) == Some(source) {
                        return Ok::<_, TransactionCommitError>(*old);
                    }
                }
            }
            Ok(source)
        }).map_err(custody::execution_stop)?.0?;
        let owner = self
            .binding
            .owners
            .recorded_owner(execution, family, original, partition)?
            .ok_or_else(|| {
                part::reject(
                    crate::publication::data::AuthoredRetirementRefusal::ExtraCoupledOwner,
                    source,
                )
            })?;
        if let RelationalDeclaredCoupledOwner::Surviving(previous) = &owner {
            execution
                .run_native_with_checkpoint(|context| {
                    part::require_unseen_route(context, &self.parts, &previous.origin.meaning)
                })
                .map_err(custody::execution_stop)?
                .0?;
        }
        Ok(owner)
    }

    pub fn pin_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>, TransactionCommitError>
    {
        RelationalPinnedRead::capture_applied(
            execution,
            self.runtime,
            &self.binding.owners.before,
            self.identity,
            self.parts.len() as u64,
            self.mutation.state(),
            self.mutation.candidate_symbols(),
            &self.mutation.outcome.effect.publication.changed_records,
            self.aggregate_read(execution)?,
        )
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
            || installed.runtime_instance_id != self.basis.identity().runtime_instance_id()
            || owners.before != self.basis
            || self.intent.materialization_mode().is_some()
            || !self.merge_parent_bases.is_empty()
            || retirement_admission
                .as_ref()
                .map_or(!owners.retirements.is_empty(), |admission| {
                    admission.publication_count() != owners.retirements.len()
                        || owners.retirements.iter().any(|prior| {
                            !installed
                                .authority
                                .admits_authored_output_retirement(admission, prior.authority())
                        })
                })
        {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                owners.anchor,
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
