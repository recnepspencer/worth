use bank_domain::{
    estate::{EstateAction, EstateCapabilityDelegationRequest},
    schema::*,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::{
        WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
        WorthQueryApplicationIdempotencyBinding,
        WorthQueryApplicationOperationInvariantProjectionReader,
        WorthQueryDelegationActivationProgram, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

use super::authorization::authorize_target;

pub(super) type DelegationAccess = WorthQueryAdmittedApplicationCapabilityAccess<
    BankSchema,
    DelegateEstateCapability,
    DelegateEstateCapabilityOperation,
    EstateAction,
>;
pub(super) type AdmittedDelegation = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    DelegateEstateCapabilityOperation,
    EstateAction,
    EstateCase,
>;
type DelegationProgram = WorthQueryDelegationActivationProgram<
    BankSchema,
    DelegateEstateCapabilityOperation,
    EstateAction,
    EstateCase,
>;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum BankCapabilityDelegationProjectionDenial {
    EntityResolution(crate::BankEntityResolutionDenial),
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

impl std::fmt::Display for BankCapabilityDelegationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankCapabilityDelegationProjectionDenial {}

impl BankIdentityRuntime {
    pub fn delegate_estate_capability(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let command = delegation_command(action)?;
        let admission = self.admit_delegation(principal, action, command.child, request)?;
        if let Some(outcome) =
            super::super::idempotency::resolve_admitted_idempotency(self, &admission, idempotency)?
        {
            return Ok(outcome);
        }
        let program = self.materialize_delegation(admission, command.child)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_capability_delegation(program, idempotency)
            .into())
    }

    fn admit_delegation(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        child: EstateCapabilityDelegationRequest,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedDelegation, BankEstateProgressionDenial> {
        let application = self.application_runtime();
        let capability = application
            .installed_schema()
            .capability(
                DelegateEstateCapability::reference(),
                DelegateEstateCapabilityOperation::reference(),
            )
            .map_err(BankEstateProgressionDenial::from_capability_installation)?;
        let selected = self
            .select_current_product()
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let operation = application
            .installed_schema()
            .installed_operation(DelegateEstateCapabilityOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        authorize_target(self, access, &operation, child)
    }

    fn materialize_delegation(
        &self,
        admission: AdmittedDelegation,
        child: EstateCapabilityDelegationRequest,
    ) -> Result<DelegationProgram, BankEstateProgressionDenial> {
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                project_delegation(reader, estate, child)
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (result, projection, _) = projected.into_parts();
        result.map_err(BankEstateProgressionDenial::CapabilityDelegationProjection)?;
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        reads
            .complete_projected_dependencies()?
            .materialize_capability_delegation_program()
            .map_err(BankEstateProgressionDenial::from_attempt)
    }
}

fn project_delegation(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        DelegateEstateCapabilityOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    child: EstateCapabilityDelegationRequest,
) -> Result<(), BankCapabilityDelegationProjectionDenial> {
    let branch = reader.resolve_entity(BranchIdentityField::reference(), child.scope.branch)?;
    let institution = reader.resolve_entity(
        InstitutionIdentityField::reference(),
        child.scope.institution,
    )?;
    reader.require_decision_relation(EstateBranch::reference(), estate, &branch)?;
    reader.require_decision_relation(BranchInstitution::reference(), &branch, &institution)?;
    if let Some(account) = child.scope.account {
        let account = reader.resolve_entity(AccountIdentity::reference(), account)?;
        reader.require_decision_relation(EstateAccount::reference(), estate, &account)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct DelegationCommand {
    child: EstateCapabilityDelegationRequest,
}

fn delegation_command(
    action: EstateAction,
) -> Result<DelegationCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::DelegateCapability { child, .. } => Ok(DelegationCommand { child }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "DelegateEstateCapabilityOperation",
        )),
    }
}

impl From<WorthQueryEntityResolutionDenial> for BankCapabilityDelegationProjectionDenial {
    fn from(value: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(value.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankCapabilityDelegationProjectionDenial {
    fn from(value: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            value.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial>
    for BankCapabilityDelegationProjectionDenial
{
    fn from(value: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            value.kind(),
        ))
    }
}
