use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    BankSchema, EstateCaseIdentityField, RequestEstateEmergencyAccessCapability,
    RequestEstateEmergencyAccessOperation,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;

use super::{
    idempotency::{elevation_binding, EstateElevationTransition},
    BankEstateElevationRequestOutcome, BankEstateProgressionDenial,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn request_estate_emergency_access_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateElevationRequestOutcome, BankEstateProgressionDenial> {
        let idempotency =
            elevation_binding(idempotency_key, EstateElevationTransition::Request, action)?;
        self.request_estate_emergency_access(principal, action, idempotency, request)
    }

    pub fn request_estate_emergency_access(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateElevationRequestOutcome, BankEstateProgressionDenial> {
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                RequestEstateEmergencyAccessCapability::reference(),
                RequestEstateEmergencyAccessOperation::reference(),
            )
            .map_err(BankEstateProgressionDenial::from_capability_installation)?;
        let selected = self
            .select_current_product()
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let operation = self
            .application_runtime()
            .installed_schema()
            .installed_operation(RequestEstateEmergencyAccessOperation::reference())
            .map_err(BankEstateProgressionDenial::from_operation_installation)?;
        let admission = self
            .application_runtime()
            .authorize_elevation_request(
                access,
                &operation,
                TypedMutationPreconditions::<
                    BankSchema,
                    RequestEstateEmergencyAccessOperation,
                    bank_domain::schema::EstateCase,
                >::default(),
            )
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                reader.require_decision_field(estate, EstateCaseIdentityField::reference())
            })
            .map_err(BankEstateProgressionDenial::from_projection)?;
        let (field_result, projection, _) = projected.into_parts();
        field_result.map_err(BankEstateProgressionDenial::from_decision_projection)?;
        let program = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)?
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)?
            .materialize_elevation_request_program()
            .map_err(BankEstateProgressionDenial::from_attempt)?;
        Ok(BankEstateElevationRequestOutcome::from_query(
            self.application_runtime()
                .compare_and_commit_elevation_request(program, idempotency),
        ))
    }
}
