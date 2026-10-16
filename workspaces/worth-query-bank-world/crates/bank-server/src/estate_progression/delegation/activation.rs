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
        WorthQueryDelegationActivationProgram, WorthQueryInvariantEntityIdentity,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

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

mod projection_denial;
#[cfg(test)]
mod tests;
use projection_denial::child_selection_denial;
pub use projection_denial::BankCapabilityDelegationProjectionDenial;

impl BankIdentityRuntime {
    /// Retains one exact delegation command. Fresh requests advance its native owner.
    #[allow(
        clippy::result_large_err,
        reason = "Capture refusal returns the original action, key, preparation owner and typed denial without another allocation"
    )]
    pub fn retain_estate_capability_delegation<'runtime>(
        &'runtime self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<super::BankDelegationCommand<'runtime>, super::BankDelegationCaptureFailure<'runtime>>
    {
        let prepare: super::BankDelegationPreparation<'runtime> =
            Box::new(move |phase, admission| {
                let child = delegation_command(action)?.child;
                self.materialize_delegation(phase, admission, child)
            });
        let child = match delegation_command(action) {
            Ok(command) => command.child,
            Err(denial) => {
                return Err((
                    action,
                    key,
                    prepare,
                    WorthQueryApplicationCapabilityDelegationDenial::Preparation(denial),
                ))
            }
        };
        self.retain_delegation_target(principal, action, child, key, request, prepare)
    }

    #[cfg(test)]
    fn admit_delegation(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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
            .select_current_product(phase)
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(phase, principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let operation = application
            .installed_schema()
            .installed_operation(DelegateEstateCapabilityOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        authorize_target(phase, self, access, &operation, child)
    }

    #[allow(
        clippy::result_large_err,
        reason = "Capture refusal returns the original action, key, preparation owner and typed denial without another allocation"
    )]
    fn retain_delegation_target<'runtime>(
        &'runtime self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        child: EstateCapabilityDelegationRequest,
        key: BankIdempotencyKey,
        request: &WorthQueryRequestScope,
        prepare: super::BankDelegationPreparation<'runtime>,
    ) -> Result<super::BankDelegationCommand<'runtime>, super::BankDelegationCaptureFailure<'runtime>>
    {
        macro_rules! execute {
            ($capability:ty, $operation:ty) => {{
                self.request(principal, request)
                    .retain_capability_delegation_in_program(
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
                        prepare,
                    )
            }};
        }
        match (child.scope.operation, child.scope.purpose) {
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
            _ => Err((
                action,
                key,
                prepare,
                WorthQueryApplicationCapabilityDelegationDenial::Preparation(
                    BankEstateProgressionDenial::CommandInput("delegated capability target"),
                ),
            )),
        }
    }

    fn materialize_delegation(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        admission: AdmittedDelegation,
        child: EstateCapabilityDelegationRequest,
    ) -> Result<DelegationProgram, BankEstateProgressionDenial> {
        let projected = self
            .invariant_projection()
            .project_admitted_operation(
                phase,
                &admission,
                |reader, estate| project_delegation(reader, estate, child),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (result, projection, _) = projected.into_parts();
        result.map_err(BankEstateProgressionDenial::CapabilityDelegationProjection)?;
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(
                phase,
                admission,
                projection,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        reads
            .complete_projected_dependencies(
                phase,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )?
            .materialize_capability_delegation_program(
                phase,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
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
    // The child id names no grant. Its absence is a fact of the decision, so
    // a delegation prepared beside one that committed the same id goes stale.
    let named = reader
        .decision_select_entities(CapabilityGrantIdentityField::reference(), child.id, 1)
        .map_err(child_selection_denial)?;
    if !named.is_empty() {
        return Err(BankCapabilityDelegationProjectionDenial::ChildGrantExists);
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
