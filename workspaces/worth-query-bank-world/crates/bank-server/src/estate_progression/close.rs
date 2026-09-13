use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, RevokeEstateEmergencyAccessCapability, RevokeEstateEmergencyAccessOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::WorthQueryApplicationIdempotencyBinding,
};

use super::{
    idempotency::{elevation_binding, EstateElevationTransition},
    lifecycle_facts::seal_close_lifecycle_facts,
    BankApprovedEstateElevation, BankEstateElevationCloseOutcome, BankEstateProgressionDenial,
    BankEstateProgressionFailure,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn revoke_estate_emergency_access_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        approved: BankApprovedEstateElevation,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateElevationCloseOutcome,
        BankEstateProgressionFailure<BankApprovedEstateElevation>,
    > {
        let idempotency =
            match elevation_binding(idempotency_key, EstateElevationTransition::Revoke, action) {
                Ok(idempotency) => idempotency,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(denial, approved));
                }
            };
        self.revoke_estate_emergency_access_retaining(
            principal,
            approved,
            action,
            idempotency,
            request,
        )
    }

    pub fn revoke_estate_emergency_access(
        &self,
        principal: &BankAuthenticatedPrincipal,
        approved: BankApprovedEstateElevation,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateElevationCloseOutcome, BankEstateProgressionDenial> {
        self.revoke_estate_emergency_access_retaining(
            principal,
            approved,
            action,
            idempotency,
            request,
        )
        .map_err(BankEstateProgressionFailure::into_denial)
    }

    fn revoke_estate_emergency_access_retaining(
        &self,
        principal: &BankAuthenticatedPrincipal,
        approved: BankApprovedEstateElevation,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateElevationCloseOutcome,
        BankEstateProgressionFailure<BankApprovedEstateElevation>,
    > {
        let EstateAction::RevokeEmergencyAccess { access, .. } = action else {
            return Err(BankEstateProgressionFailure::retained(
                BankEstateProgressionDenial::CommandInput("RevokeEstateEmergencyAccessOperation"),
                approved,
            ));
        };
        let capability = match self.application_runtime().installed_schema().capability(
            RevokeEstateEmergencyAccessCapability::reference(),
            RevokeEstateEmergencyAccessOperation::reference(),
        ) {
            Ok(capability) => capability,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_capability_installation(denial),
                    approved,
                ));
            }
        };
        let selected = match self.select_current_product() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_product_selection(denial),
                    approved,
                ));
            }
        };
        let access_authority =
            match selected.admit_capability_access(principal.query(), &capability, action, request)
            {
                Ok(access) => access,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(
                        BankEstateProgressionDenial::from_authorization(denial),
                        approved,
                    ));
                }
            };
        let operation = match self
            .application_runtime()
            .installed_schema()
            .installed_operation(RevokeEstateEmergencyAccessOperation::reference())
        {
            Ok(operation) => operation,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_operation_installation(denial),
                    approved,
                ));
            }
        };
        let admission = match self.application_runtime().authorize_elevation_close(
            approved.into_query(),
            access_authority,
            &operation,
            TypedMutationPreconditions::<
                BankSchema,
                RevokeEstateEmergencyAccessOperation,
                bank_domain::schema::EstateCase,
            >::default(),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                let mapped = BankEstateProgressionDenial::from_close_authorization_ref(&denial);
                return Err(BankEstateProgressionFailure::retained(
                    mapped,
                    BankApprovedEstateElevation::from_query(denial.into_approved()),
                ));
            }
        };
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                seal_close_lifecycle_facts(reader, access, estate)
            })
            .map_err(BankEstateProgressionDenial::from_projection)
            .map_err(BankEstateProgressionFailure::consumed)?;
        let (lifecycle_result, projection, _) = projected.into_parts();
        lifecycle_result
            .map_err(BankEstateProgressionDenial::LifecycleProjection)
            .map_err(BankEstateProgressionFailure::consumed)?;
        let program = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?
            .materialize_elevation_close_program()
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?;
        Ok(BankEstateElevationCloseOutcome::from_query(
            self.application_runtime()
                .compare_and_commit_elevation_close(program, idempotency),
        ))
    }
}
