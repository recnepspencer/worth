use bank_domain::{
    estate::{
        EstateAction, EstateCapabilityDelegationRequest, EstateCapabilityOperation,
        EstateCapabilityPurpose,
    },
    proposals::BankIdempotencyKey,
    schema::*,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::WorthQueryApplicationCapabilityDelegationDenial,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation,
        WorthQueryApplicationOperationInvariantProjectionReader,
        WorthQueryDelegationActivationProgram, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{
    BankAuthenticatedPrincipal, BankCommitDenialKind, BankCommitDenialStage, BankIdentityRuntime,
    BankMutationCommitOutcome,
};

#[cfg(test)]
use super::authorization::authorize_target;

#[cfg(test)]
pub(super) type DelegationAccess =
    worth_query_host::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess<
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
    pub fn delegate_estate_capability_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let command = delegation_command(action)?;
        self.execute_delegation_target(principal, action, command.child, key, request)
    }

    #[cfg(test)]
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

    fn execute_delegation_target(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        child: EstateCapabilityDelegationRequest,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        macro_rules! execute {
            ($capability:ty, $operation:ty) => {{
                self.request(principal, request)
                    .execute_capability_delegation_in_program(
                        self.application_program(),
                        BankPrincipalBinding::reference(),
                        DelegateEstateCapability::reference(),
                        <$capability>::reference(),
                        <$operation>::reference(),
                        DelegateEstateCapabilityOperation::reference(),
                        action,
                        key,
                        TypedMutationPreconditions::<
                            BankSchema,
                            DelegateEstateCapabilityOperation,
                            EstateCase,
                        >::default(),
                        |admission| self.materialize_delegation(admission, child),
                    )
            }};
        }
        let outcome = match (child.scope.operation, child.scope.purpose) {
            (
                EstateCapabilityOperation::NotifyDeath,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(NotifyDeathEstateCapability, NotifyDeathEstateOperation)
            }
            (
                EstateCapabilityOperation::RetransmitDeathNotice,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(
                    RetransmitDeathNoticeEstateCapability,
                    RetransmitDeathNoticeEstateOperation
                )
            }
            (
                EstateCapabilityOperation::FreezeAccount,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(FreezeEstateAccountCapability, FreezeEstateAccountOperation)
            }
            (
                EstateCapabilityOperation::OpenEstateCase,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(OpenEstateCaseCapability, OpenEstateCaseOperation)
            }
            (
                EstateCapabilityOperation::RecognizeExecutor,
                EstateCapabilityPurpose::LegalCompliance,
            ) => {
                execute!(
                    RecognizeEstateExecutorCapability,
                    RecognizeEstateExecutorOperation
                )
            }
            (
                EstateCapabilityOperation::ReleaseEstate,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(ReleaseEstateCapability, ReleaseEstateOperation)
            }
            (
                EstateCapabilityOperation::DisburseEstate,
                EstateCapabilityPurpose::EstateDisbursement,
            ) => {
                execute!(DisburseEstateCapability, DisburseEstateOperation)
            }
            (
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::EstateAdministration,
            ) => {
                execute!(
                    ViewEstateAdministrationCapability,
                    ViewRestrictedEstateOperation
                )
            }
            (
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::IdentityVerification,
            ) => {
                execute!(
                    ViewEstateIdentityVerificationCapability,
                    ViewRestrictedEstateOperation
                )
            }
            (
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::LegalCompliance,
            ) => {
                execute!(
                    ViewEstateLegalComplianceCapability,
                    ViewRestrictedEstateOperation
                )
            }
            (
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::EmergencyProtection,
            ) => {
                execute!(
                    ViewEstateEmergencyProtectionCapability,
                    ViewRestrictedEstateOperation
                )
            }
            (
                EstateCapabilityOperation::ViewRestrictedEstate,
                EstateCapabilityPurpose::MandatoryReview,
            ) => {
                execute!(
                    ViewEstateMandatoryReviewCapability,
                    ViewRestrictedEstateOperation
                )
            }
            _ => {
                return Err(BankEstateProgressionDenial::CommandInput(
                    "delegated capability target",
                ))
            }
        };
        match outcome {
            Ok(outcome) => Ok(outcome.into()),
            Err(WorthQueryApplicationCapabilityDelegationDenial::IdempotencyIntentDrift) => {
                Ok(BankMutationCommitOutcome::Denied {
                    kind: BankCommitDenialKind::IdempotencyIntentDrift,
                    stage: BankCommitDenialStage::Idempotency,
                })
            }
            Err(denial) => Err(map_delegation_denial(denial)),
        }
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

fn map_delegation_denial(
    denial: WorthQueryApplicationCapabilityDelegationDenial<BankEstateProgressionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationCapabilityDelegationDenial as Query;
    match denial {
        Query::Program(denial) => BankEstateProgressionDenial::ProgramAction(denial),
        Query::ProgramMismatch => BankEstateProgressionDenial::ProgramMismatch,
        Query::PrincipalBindingInstallation(denial) => {
            BankEstateProgressionDenial::PrincipalBindingInstallation(denial)
        }
        Query::CapabilityInstallation(denial) => {
            BankEstateProgressionDenial::from_capability_installation(denial)
        }
        Query::OperationInstallation(denial) => {
            BankEstateProgressionDenial::from_operation_installation(denial)
        }
        Query::ProductSelection(denial) => {
            BankEstateProgressionDenial::from_product_selection(denial)
        }
        Query::PrincipalResolution(denial) => {
            BankEstateProgressionDenial::PrincipalResolution(denial)
        }
        Query::PrincipalIdentityEncoding(denial) => {
            BankEstateProgressionDenial::PrincipalIdentityEncoding(denial)
        }
        Query::Authorization(denial) => BankEstateProgressionDenial::from_authorization(denial),
        Query::Idempotency(denial) => BankEstateProgressionDenial::from_idempotency(denial),
        Query::IdempotencyIntentDrift => BankEstateProgressionDenial::IdempotencyIntentDrift,
        Query::Preparation(denial) => denial,
    }
}
