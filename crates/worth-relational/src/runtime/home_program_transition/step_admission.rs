//! Admit an exact original Home step before entering shared transaction assembly.
use super::*;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionCaller, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation, HomeProgramTransitionStepRole,
};

/// Declarative selection; the concrete Home grant and envelope authenticate it.
pub enum RelationalHomeProgramStepContracts<'contracts> {
    BeforeTarget,
    Target {
        contracts: &'contracts PreparedRelationalHomeProgramContracts,
        transition: Option<(
            crate::schema::data::ProposedSchemaTransition,
            Option<crate::schema::data::SchemaReconciliationPolicy>,
        )>,
    },
    AfterTarget,
}

#[derive(Debug)]
pub(crate) struct AdmittedRelationalHomeProgramStep {
    pub(crate) step: HomeProgramTransitionStepAssociation,
    pub(crate) admission: worth_proof::AdmittedHomeProgramTransitionStep<
        crate::branch::RelationalBranchBasisAdmissionIdentity,
    >,
    pub(crate) commitment: [u8; 32],
    pub(crate) association: [u8; 32],
    pub(crate) index_budget: crate::indexes::data::DerivedIndexMaintenanceBudget,
    _memory: Option<ExecutionMemoryReservation>,
}
impl AdmittedRelationalHomeProgramStep {
    pub(crate) fn admits_association(&self, bytes: &[u8]) -> bool {
        HomeProgramTransitionCaller::recognizes(bytes)
            && self.association == <[u8; 32]>::from(Sha256::digest(bytes))
    }
}

impl RelationalRuntime {
    /// Every transition step uses this same authenticated source-bound receiver.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_home_program_step(
        &self,
        request: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        contracts: RelationalHomeProgramStepContracts<'_>,
        plan: &HomeProgramTransitionPlanDescription,
        step: HomeProgramTransitionStepAssociation,
        original: &CompositeAttemptAssociation,
        association: &crate::mvcc::RelationalCallerAssociation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            crate::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        index_budget: crate::indexes::data::DerivedIndexMaintenanceBudget,
    ) -> Result<crate::mvcc::BranchBoundRelationalTransaction, RelationalHomeProgramContractDenial>
    {
        use RelationalHomeProgramContractDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let commitment = plan.step_commitment(step, original).map_err(Denial::Plan)?;
        let medium = self.durability.medium().ok_or(Denial::UnboundHome)?;
        let observed: [u8; 32] = Sha256::digest(association.as_bytes()).into();
        medium
            .home_transition_authority()
            .ok_or(Denial::UnboundHome)?
            .admit_step(admission, &commitment, &observed)
            .map_err(Denial::Admission)?;
        if admission.source() != basis.admission_identity() {
            return Err(Denial::SourceBasisMismatch);
        }
        let envelope = HomeProgramTransitionCaller::read(association.as_bytes())
            .map_err(Denial::CallerEnvelope)?;
        let role = plan.step_role(step.step()).map_err(Denial::Plan)?;
        if envelope.step() != step
            || envelope.role() != role
            || envelope.catalog() != plan.target_meaning().catalog()
            || envelope.commitment() != commitment
        {
            return Err(Denial::TargetStepMismatch);
        }
        match (&contracts, role) {
            (
                RelationalHomeProgramStepContracts::BeforeTarget,
                HomeProgramTransitionStepRole::BeforeTarget,
            ) => {}
            (
                RelationalHomeProgramStepContracts::Target { contracts, .. },
                HomeProgramTransitionStepRole::Target,
            ) => {
                if contracts.catalog_digest() != envelope.catalog()
                    || contracts.schema_digest() != plan.target_meaning().schema()
                    || contracts.target_meaning(request, step.step())?.carrier()
                        != plan.target_meaning().carrier()
                {
                    return Err(Denial::TargetCatalogMismatch);
                }
            }
            (
                RelationalHomeProgramStepContracts::AfterTarget,
                HomeProgramTransitionStepRole::AfterTarget,
            ) => {
                let inherited = basis
                    .inner
                    .root
                    .execution_contracts
                    .get()
                    .ok_or(Denial::TargetCatalogMismatch)?;
                if inherited.digest != envelope.catalog()
                    || inherited.schema.authority_digest() != plan.target_meaning().schema().bytes()
                    || inherited
                        .schema
                        .recorded_image(request)
                        .map_err(Denial::SchemaImage)?
                        .carrier_digest()
                        != plan.target_meaning().carrier().bytes()
                {
                    return Err(Denial::TargetCatalogMismatch);
                }
            }
            (
                RelationalHomeProgramStepContracts::BeforeTarget,
                HomeProgramTransitionStepRole::Target | HomeProgramTransitionStepRole::AfterTarget,
            )
            | (
                RelationalHomeProgramStepContracts::Target { .. },
                HomeProgramTransitionStepRole::BeforeTarget
                | HomeProgramTransitionStepRole::AfterTarget,
            )
            | (
                RelationalHomeProgramStepContracts::AfterTarget,
                HomeProgramTransitionStepRole::BeforeTarget | HomeProgramTransitionStepRole::Target,
            ) => {
                return Err(Denial::TargetStepMismatch);
            }
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AdmittedRelationalHomeProgramStep>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        let admitted = Arc::new(AdmittedRelationalHomeProgramStep {
            step,
            admission: admission.clone(),
            commitment,
            association: observed,
            index_budget,
            _memory: memory,
        });
        match contracts {
            RelationalHomeProgramStepContracts::Target {
                contracts,
                transition,
            } => self
                .prepare_home_target_transaction(request, basis, contracts, transition, admitted),
            RelationalHomeProgramStepContracts::BeforeTarget
            | RelationalHomeProgramStepContracts::AfterTarget => self
                .begin_branch_transaction_with_scope(
                    basis,
                    crate::mvcc::RelationalTransactionIntent::ordinary(),
                    crate::mvcc::RelationalOperationControl::uninterrupted(),
                    Some(admitted),
                    None,
                )
                .map_err(Denial::Transaction),
        }
    }
}
