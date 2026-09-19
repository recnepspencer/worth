use std::sync::{Arc, Mutex, TryLockError};

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::SignalBranchBasisObservationDenial;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionResolver, InstalledSignalConditionalContract,
    SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure,
    SignalConditionalExecutionRequest, SignalConditionalServiceContractBinding,
};
use crate::data::error::SignalError;
use crate::data::output::NodeEvaluationResult;
use crate::data::proof::SignalInvalidationExecutionReceipt;
use crate::data::retained_storage::{
    SignalConditionalRetentionDenial, SignalConditionalRetentionReservation,
};
use crate::logic::transaction::SignalObservationAdmissionDenial;
use worth_proof::{AdmittedConditionalSourceObservation, ConditionalEvaluationSource};

use super::issuance::SignalConditionalServiceAuthority;
use super::{SignalConditionalExecutionPort, SignalRetainedExecutionBasis};

pub(in crate::branch::owner_services) struct SignalConditionalExecutionSlot {
    pub(in crate::branch::owner_services) partition:
        crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,
}

pub(in crate::branch::owner_services) struct SignalConditionalEvaluationState {
    pub(in crate::branch::owner_services) admission_custody: SignalConditionalRetentionReservation,
    pub(in crate::branch::owner_services) slot: Option<SignalConditionalExecutionSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalConditionalServiceExecutionRequest {
    pub(super) attempt: u64,
    pub(super) force_on_demand: bool,
}

impl SignalConditionalServiceExecutionRequest {
    pub const fn new(attempt: u64) -> Self {
        Self {
            attempt,
            force_on_demand: false,
        }
    }

    pub const fn force_on_demand(mut self) -> Self {
        self.force_on_demand = true;
        self
    }
}

/// Signal-issued proof that one exact source observation and installed
/// contract were admitted against this service's selected branch basis.
pub struct SignalConditionalEvaluationAdmission {
    pub(super) service_authority: Arc<SignalConditionalServiceAuthority>,
    pub(in crate::branch::owner_services) contract_binding: SignalConditionalServiceContractBinding,
    pub(in crate::branch::owner_services) contract: InstalledSignalConditionalContract,
    pub(super) source: Arc<SignalConditionalEvaluationSourceEvidence>,
    pub(super) execution_identity: Arc<str>,
    pub(in crate::branch::owner_services) retained_basis: Arc<SignalRetainedExecutionBasis>,
    pub(super) execution: Mutex<SignalConditionalEvaluationState>,
}

impl std::fmt::Debug for SignalConditionalEvaluationAdmission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SignalConditionalEvaluationAdmission")
            .field("posture", &"signal-admitted")
            .finish_non_exhaustive()
    }
}

pub struct SignalConditionalEvaluationBindingEvidence {
    _service_authority: Arc<SignalConditionalServiceAuthority>,
    source: Arc<SignalConditionalEvaluationSourceEvidence>,
    _execution_identity: Arc<str>,
}

impl SignalConditionalEvaluationBindingEvidence {
    pub fn source(&self) -> &SignalConditionalEvaluationSourceEvidence {
        &self.source
    }
}

/// Signal-sealed evidence for the source posture used by one evaluation.
/// Private fields prevent a descriptive source identity from becoming proof.
#[derive(Debug)]
pub struct SignalConditionalEvaluationSourceEvidence {
    source: SignalConditionalEvaluationSourceBinding,
}

#[derive(Debug)]
enum SignalConditionalEvaluationSourceBinding {
    NoRelationalSource {
        projection: Arc<str>,
    },
    AdmittedRelationalSource {
        source: AdmittedConditionalSourceObservation,
    },
}

impl SignalConditionalEvaluationSourceEvidence {
    pub fn projection(&self) -> &str {
        match &self.source {
            SignalConditionalEvaluationSourceBinding::NoRelationalSource { projection } => {
                projection
            }
            SignalConditionalEvaluationSourceBinding::AdmittedRelationalSource { source } => {
                source.projection()
            }
        }
    }

    pub fn admitted_relational_source(&self) -> Option<&AdmittedConditionalSourceObservation> {
        match &self.source {
            SignalConditionalEvaluationSourceBinding::NoRelationalSource { .. } => None,
            SignalConditionalEvaluationSourceBinding::AdmittedRelationalSource { source } => {
                Some(source)
            }
        }
    }

    pub(super) fn retained_representation_bytes(&self) -> usize {
        match &self.source {
            SignalConditionalEvaluationSourceBinding::NoRelationalSource { projection } => {
                projection
                    .len()
                    .saturating_add(4 * std::mem::size_of::<usize>())
            }
            SignalConditionalEvaluationSourceBinding::AdmittedRelationalSource { source } => {
                source.retained_representation_bytes()
            }
        }
    }

    pub(super) fn no_relational_source(projection: Arc<str>) -> Self {
        Self {
            source: SignalConditionalEvaluationSourceBinding::NoRelationalSource { projection },
        }
    }

    pub(super) fn from_admitted_relational_source(
        source: AdmittedConditionalSourceObservation,
    ) -> Self {
        Self {
            source: SignalConditionalEvaluationSourceBinding::AdmittedRelationalSource { source },
        }
    }
}

#[derive(Debug)]
pub enum SignalConditionalServiceExecutionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    StaleBasisAdmission,
    DefinitionReadmissionRequired,
    DefinitionMismatch,
    NestedOperationScopeMismatch,
    MissingSourceEvidence,
    UnexpectedSourceEvidence,
    SourceAuthorityMismatch,
    EvaluationIdentityExhausted,
    AdmissionCapacityExhausted,
    AdmissionUnavailable,
    SlotBusy,
    SlotPoisoned,
    SlotAdmission(SignalError),
    ObservationAdmission(SignalObservationAdmissionDenial),
    UnconsumedUnwind,
}

pub struct SignalConditionalServiceCompletion {
    decision: Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>,
    observation: Result<Option<SignalInvalidationExecutionReceipt>, SignalError>,
    binding: SignalConditionalEvaluationBindingEvidence,
    slot_reused: bool,
}

impl SignalConditionalServiceCompletion {
    pub fn binding(&self) -> &SignalConditionalEvaluationBindingEvidence {
        &self.binding
    }

    pub const fn slot_reused(&self) -> bool {
        self.slot_reused
    }

    pub(in crate::branch::owner_services) fn with_slot_reuse(mut self, slot_reused: bool) -> Self {
        self.slot_reused = slot_reused;
        self
    }

    pub fn into_parts(
        self,
    ) -> (
        Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure>,
        Result<Option<SignalInvalidationExecutionReceipt>, SignalError>,
    ) {
        (self.decision, self.observation)
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn admit_evaluation(
        &self,
        contract: &InstalledSignalConditionalContract,
        source: ConditionalEvaluationSource,
    ) -> Result<SignalConditionalEvaluationAdmission, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;

        match &source {
            ConditionalEvaluationSource::NoRelationalSource
                if !contract.dependency_aspects().is_empty() =>
            {
                return Err(Denial::MissingSourceEvidence);
            }
            ConditionalEvaluationSource::AdmittedRelationalSource(_)
                if contract.dependency_aspects().is_empty() =>
            {
                return Err(Denial::UnexpectedSourceEvidence);
            }
            ConditionalEvaluationSource::AdmittedRelationalSource(source)
                if !self.source_authority.admits(source) =>
            {
                return Err(Denial::SourceAuthorityMismatch);
            }
            _ => {}
        }
        let (contract_binding, retained_basis) = self.admit_contract_binding(contract)?;
        let ordinal = self
            .next_evaluation_ordinal
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .map_err(|_| Denial::EvaluationIdentityExhausted)?;
        let execution_identity = super::evaluation_identity::evaluation_identity(ordinal);
        let source = Arc::new(match source {
            ConditionalEvaluationSource::NoRelationalSource => {
                SignalConditionalEvaluationSourceEvidence::no_relational_source(Arc::from(format!(
                    "signal-no-relational-source:{}:{}:{}:{}:{}",
                    contract.graph_instance_id(),
                    contract.node().index(),
                    contract.generation(),
                    contract.occurrence(),
                    execution_identity,
                )))
            }
            ConditionalEvaluationSource::AdmittedRelationalSource(source) => {
                SignalConditionalEvaluationSourceEvidence::from_admitted_relational_source(source)
            }
        });
        let charge = super::admission_retention::evaluation_admission_charge(
            contract,
            &source,
            &execution_identity,
        )
        .map_err(|_| Denial::AdmissionCapacityExhausted)?;
        let admission_custody = retained_basis
            .reserve_evaluation_admission(charge)
            .map_err(map_retention_denial)?;
        Ok(SignalConditionalEvaluationAdmission {
            service_authority: Arc::clone(&self.authority),
            contract_binding,
            contract: contract.clone(),
            source,
            execution_identity,
            retained_basis,
            execution: Mutex::new(SignalConditionalEvaluationState {
                admission_custody,
                slot: None,
            }),
        })
    }

    pub fn execute(
        &self,
        evaluation: &SignalConditionalEvaluationAdmission,
        request: SignalConditionalServiceExecutionRequest,
        condition: &mut impl InstalledSignalConditionResolver,
        comparator: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;

        if !Arc::ptr_eq(&self.authority, &evaluation.service_authority) {
            return Err(Denial::DefinitionMismatch);
        }
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        let mut evaluation_state = match evaluation.execution.try_lock() {
            Ok(execution) => execution,
            Err(TryLockError::WouldBlock) => return Err(Denial::SlotBusy),
            Err(TryLockError::Poisoned(_)) => return Err(Denial::SlotPoisoned),
        };
        let mut kernel_request = SignalConditionalExecutionRequest::new(
            &evaluation.contract,
            evaluation.source.projection(),
            evaluation.execution_identity.as_ref(),
            request.attempt,
        );
        if request.force_on_demand {
            kernel_request = kernel_request.force_on_demand();
        }
        let slot_reused = evaluation_state.slot.is_some();
        let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.execute_conditional(
                &admission,
                &self.basis,
                &self.definition,
                evaluation,
                &mut evaluation_state,
                kernel_request,
                condition,
                comparator,
                compute,
            )
        }));
        match execution {
            Ok(Ok(completion)) => Ok(completion.with_slot_reuse(slot_reused)),
            Ok(Err(denial)) => Err(denial),
            Err(payload) => {
                drop(evaluation_state);
                std::panic::resume_unwind(payload)
            }
        }
    }
}

fn map_retention_denial(
    denial: SignalConditionalRetentionDenial,
) -> SignalConditionalServiceExecutionDenial {
    match denial {
        SignalConditionalRetentionDenial::CapacityExhausted => {
            SignalConditionalServiceExecutionDenial::AdmissionCapacityExhausted
        }
        SignalConditionalRetentionDenial::Closed => {
            SignalConditionalServiceExecutionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalConditionalRetentionDenial::Poisoned
        | SignalConditionalRetentionDenial::InvalidTransfer => {
            SignalConditionalServiceExecutionDenial::AdmissionUnavailable
        }
    }
}

impl SignalConditionalServiceCompletion {
    pub(in crate::branch::owner_services) fn from_partition(
        completion: crate::data::graph::storage::evaluation_partition::SignalPartitionConditionalCompletion,
        evaluation: &SignalConditionalEvaluationAdmission,
    ) -> Self {
        let (decision, observation, rejected) = completion.into_parts();
        drop(rejected);
        Self {
            decision,
            observation,
            binding: SignalConditionalEvaluationBindingEvidence {
                _service_authority: Arc::clone(&evaluation.service_authority),
                source: Arc::clone(&evaluation.source),
                _execution_identity: Arc::clone(&evaluation.execution_identity),
            },
            slot_reused: false,
        }
    }
}
