use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation, BankSchema,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::WorthQueryApplicationIdempotencyBinding,
};

use super::{
    idempotency::{elevation_binding, EstateElevationTransition},
    lifecycle_facts::{approval_lifecycle_identities, seal_approval_lifecycle_facts},
    BankEstateElevationApprovalOutcome, BankEstateProgressionDenial, BankEstateProgressionFailure,
    BankRequestedEstateElevation,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn approve_estate_emergency_access_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        requested: BankRequestedEstateElevation,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateElevationApprovalOutcome,
        BankEstateProgressionFailure<BankRequestedEstateElevation>,
    > {
        let idempotency =
            match elevation_binding(idempotency_key, EstateElevationTransition::Approve, action) {
                Ok(idempotency) => idempotency,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(denial, requested));
                }
            };
        self.approve_estate_emergency_access_retaining(
            principal,
            requested,
            action,
            idempotency,
            request,
        )
    }

    pub fn approve_estate_emergency_access(
        &self,
        principal: &BankAuthenticatedPrincipal,
        requested: BankRequestedEstateElevation,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateElevationApprovalOutcome, BankEstateProgressionDenial> {
        self.approve_estate_emergency_access_retaining(
            principal,
            requested,
            action,
            idempotency,
            request,
        )
        .map_err(BankEstateProgressionFailure::into_denial)
    }

    fn approve_estate_emergency_access_retaining(
        &self,
        principal: &BankAuthenticatedPrincipal,
        requested: BankRequestedEstateElevation,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateElevationApprovalOutcome,
        BankEstateProgressionFailure<BankRequestedEstateElevation>,
    > {
        let (access_identity, review_identity) =
            match approval_lifecycle_identities(requested.query()) {
                Ok(identities) => identities,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(
                        BankEstateProgressionDenial::LifecycleProjection(denial),
                        requested,
                    ));
                }
            };
        let capability = match self.application_runtime().installed_schema().capability(
            ApproveEstateEmergencyAccessCapability::reference(),
            ApproveEstateEmergencyAccessOperation::reference(),
        ) {
            Ok(capability) => capability,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_capability_installation(denial),
                    requested,
                ));
            }
        };
        let selected = match self.select_current_product() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_product_selection(denial),
                    requested,
                ));
            }
        };
        let access =
            match selected.admit_capability_access(principal.query(), &capability, action, request)
            {
                Ok(access) => access,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(
                        BankEstateProgressionDenial::from_authorization(denial),
                        requested,
                    ));
                }
            };
        let operation = match self
            .application_runtime()
            .installed_schema()
            .installed_operation(ApproveEstateEmergencyAccessOperation::reference())
        {
            Ok(operation) => operation,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_operation_installation(denial),
                    requested,
                ));
            }
        };
        let admission = match self.application_runtime().authorize_elevation_approval(
            requested.into_query(),
            access,
            &operation,
            TypedMutationPreconditions::<
                BankSchema,
                ApproveEstateEmergencyAccessOperation,
                bank_domain::schema::EstateCase,
            >::default(),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                let mapped = BankEstateProgressionDenial::from_approval_authorization_ref(&denial);
                return Err(BankEstateProgressionFailure::retained(
                    mapped,
                    BankRequestedEstateElevation::from_query(denial.into_requested()),
                ));
            }
        };
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                seal_approval_lifecycle_facts(reader, access_identity, review_identity, estate)
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
            .materialize_elevation_approval_program()
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?;
        Ok(BankEstateElevationApprovalOutcome::from_query(
            self.application_runtime()
                .compare_and_commit_elevation_approval(program, idempotency),
        ))
    }
}
