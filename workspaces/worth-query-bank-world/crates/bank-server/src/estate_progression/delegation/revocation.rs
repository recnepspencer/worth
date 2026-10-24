use bank_domain::{
    estate::{CapabilityGrantId, CapabilityGrantStatus, EstateAction},
    proposals::BankIdempotencyKey,
    schema::{
        BankPrincipalBinding, BankSchema, CapabilityEstate, CapabilityGrantIdentityField,
        CapabilityGrantStatusField, EstateCase, RevokeEstateCapability,
        RevokeEstateCapabilityOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::{
        WorthQueryAdmittedApplicationOperation,
        WorthQueryApplicationOperationInvariantProjectionReader,
        WorthQueryCapabilityRevocationProgram, WorthQueryEntityResolutionDenial,
        WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
        WorthQueryInvariantProjectionTraversalDenial,
    },
};

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

type AdmittedCapabilityRevocation = WorthQueryAdmittedApplicationOperation<
    BankSchema,
    RevokeEstateCapabilityOperation,
    EstateAction,
    EstateCase,
>;
type CapabilityRevocationProgram = WorthQueryCapabilityRevocationProgram<
    BankSchema,
    RevokeEstateCapabilityOperation,
    EstateAction,
    EstateCase,
>;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum BankCapabilityRevocationProjectionDenial {
    EntityResolution(crate::BankEntityResolutionDenial),
    MissingGrantIdentity,
    GrantIdentityMismatch,
    MissingGrantStatus,
    GrantNotActive,
    DecisionPlan(crate::BankInvariantDecisionPlanDenial),
    Traversal(crate::BankInvariantProjectionTraversalDenial),
}

impl std::fmt::Display for BankCapabilityRevocationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityResolution(denial) => denial.fmt(formatter),
            Self::MissingGrantIdentity => formatter.write_str("missing capability grant identity"),
            Self::GrantIdentityMismatch => {
                formatter.write_str("capability grant identity mismatch")
            }
            Self::MissingGrantStatus => formatter.write_str("missing capability grant status"),
            Self::GrantNotActive => formatter.write_str("capability grant is not active"),
            Self::DecisionPlan(denial) => denial.fmt(formatter),
            Self::Traversal(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for BankCapabilityRevocationProjectionDenial {}

impl BankIdentityRuntime {
    /// Retains one exact revocation command, including its original key and principal.
    #[allow(
        clippy::result_large_err,
        reason = "Capture refusal returns the original action, key, preparation owner and typed denial without another allocation"
    )]
    pub fn retain_estate_capability_revocation(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<super::BankRevocationCommand<'static>, super::BankRevocationCaptureFailure<'static>>
    {
        let projection = self.retain_invariant_projection();
        let program = self.application_program().clone();
        let prepare: super::BankRevocationPreparation<'static> =
            Box::new(move |phase, admission| {
                let grant = revocation_target_grant(action)?;
                materialize_capability_revocation(
                    program.runtime(),
                    &projection,
                    phase,
                    admission,
                    grant,
                )
            });
        self.request(principal, request)
            .retain_capability_revocation_in_program(
                self.application_program(),
                BankPrincipalBinding::reference(),
                RevokeEstateCapability::reference(),
                RevokeEstateCapabilityOperation::reference(),
                action,
                key,
                TypedMutationPreconditions::<BankSchema, RevokeEstateCapabilityOperation, EstateCase>::default(),
                prepare,
            )
    }

    #[cfg(test)]
    fn admit_capability_revocation(
        &self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<AdmittedCapabilityRevocation, BankEstateProgressionDenial> {
        let application = self.application_runtime();
        let capability = application
            .installed_schema()
            .capability(
                RevokeEstateCapability::reference(),
                RevokeEstateCapabilityOperation::reference(),
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
            .installed_operation(RevokeEstateCapabilityOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        application
            .authorize_capability_revocation(phase,
                access,
                &capability,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    RevokeEstateCapabilityOperation,
                    EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)
    }
}

fn materialize_capability_revocation(
    application: &worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        BankSchema,
    >,
    projection: &worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<BankSchema>,
    phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    admission: AdmittedCapabilityRevocation,
    expected_grant: CapabilityGrantId,
) -> Result<CapabilityRevocationProgram, BankEstateProgressionDenial> {
    let projected = projection
        .project_admitted_operation(
            phase,
            &admission,
            |reader, estate| project_active_estate_grant(reader, estate, expected_grant),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(BankEstateProgressionDenial::from_projection)?;
    let (result, projection, _) = projected.into_parts();
    result.map_err(BankEstateProgressionDenial::CapabilityRevocationProjection)?;
    let reads = application
        .begin_projected_application_read_attempt(
            phase,
            admission,
            projection,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(BankEstateProgressionDenial::from_attempt)?;
    let grant = reads
        .resolve_entity(
            phase,
            CapabilityGrantIdentityField::reference(),
            expected_grant,
        )
        .map_err(BankEstateProgressionDenial::from_attempt)?;
    reads
        .complete_projected_dependencies(
            phase,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(BankEstateProgressionDenial::from_attempt)?
        .materialize_capability_revocation_program(&grant)
        .map_err(BankEstateProgressionDenial::from_attempt)
}

fn revocation_target_grant(
    action: EstateAction,
) -> Result<CapabilityGrantId, BankEstateProgressionDenial> {
    match action {
        EstateAction::RevokeCapability { grant, .. } => Ok(grant),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "RevokeEstateCapabilityOperation",
        )),
    }
}

fn project_active_estate_grant(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RevokeEstateCapabilityOperation,
    >,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected_grant: CapabilityGrantId,
) -> Result<(), BankCapabilityRevocationProjectionDenial> {
    let grant = reader.resolve_entity(CapabilityGrantIdentityField::reference(), expected_grant)?;
    let observed_grant = reader
        .decision_field(&grant, CapabilityGrantIdentityField::reference())?
        .ok_or(BankCapabilityRevocationProjectionDenial::MissingGrantIdentity)?;
    if observed_grant != expected_grant {
        return Err(BankCapabilityRevocationProjectionDenial::GrantIdentityMismatch);
    }
    reader.require_decision_relation(CapabilityEstate::reference(), &grant, estate)?;
    let status = reader
        .decision_field(&grant, CapabilityGrantStatusField::reference())?
        .ok_or(BankCapabilityRevocationProjectionDenial::MissingGrantStatus)?;
    if status != CapabilityGrantStatus::Active {
        return Err(BankCapabilityRevocationProjectionDenial::GrantNotActive);
    }
    Ok(())
}

impl From<WorthQueryEntityResolutionDenial> for BankCapabilityRevocationProjectionDenial {
    fn from(denial: WorthQueryEntityResolutionDenial) -> Self {
        Self::EntityResolution(crate::BankEntityResolutionDenial::from_query(denial.kind()))
    }
}

impl From<WorthQueryInvariantDecisionPlanDenial> for BankCapabilityRevocationProjectionDenial {
    fn from(denial: WorthQueryInvariantDecisionPlanDenial) -> Self {
        Self::DecisionPlan(crate::BankInvariantDecisionPlanDenial::from_query(
            denial.kind(),
        ))
    }
}

impl From<WorthQueryInvariantProjectionTraversalDenial>
    for BankCapabilityRevocationProjectionDenial
{
    fn from(denial: WorthQueryInvariantProjectionTraversalDenial) -> Self {
        Self::Traversal(crate::BankInvariantProjectionTraversalDenial::from_query(
            denial.kind(),
        ))
    }
}
