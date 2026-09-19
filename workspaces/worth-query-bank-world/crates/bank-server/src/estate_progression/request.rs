use super::{BankEstateElevationRequestOutcome, BankEstateProgressionDenial};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use bank_domain::estate::EstateAction;
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    BankPrincipalBinding, BankSchema, EstateCaseIdentityField,
    RequestEstateEmergencyAccessCapability, RequestEstateEmergencyAccessOperation,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::WorthQueryApplicationElevationRequestDenial;
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

impl BankIdentityRuntime {
    pub fn request_estate_emergency_access_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateElevationRequestOutcome, BankEstateProgressionDenial> {
        Ok(BankEstateElevationRequestOutcome::from_query(
            self.request(principal, request)
                .execute_elevation_request_in_program(
                    self.application_program(),
                    BankPrincipalBinding::reference(),
                    RequestEstateEmergencyAccessCapability::reference(),
                    RequestEstateEmergencyAccessOperation::reference(),
                    action,
                    idempotency_key,
                    TypedMutationPreconditions::<
                        BankSchema,
                        RequestEstateEmergencyAccessOperation,
                        bank_domain::schema::EstateCase,
                    >::default(),
                    self.invariant_projection(),
                    |reader, estate| {
                        reader
                            .require_decision_field(estate, EstateCaseIdentityField::reference())
                            .map(|_| ())
                    },
                )
                .map_err(map_request_denial)?,
        ))
    }
}

fn map_request_denial(
    denial: WorthQueryApplicationElevationRequestDenial<
        worth_query_host::facade::primary_graph::WorthQueryInvariantDecisionPlanDenial,
    >,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationElevationRequestDenial as Query;
    match denial {
        Query::Program(denial) => BankEstateProgressionDenial::ProgramAction(denial),
        Query::ProgramMismatch => BankEstateProgressionDenial::ProgramMismatch,
        Query::PrincipalBindingInstallation(denial) => {
            BankEstateProgressionDenial::PrincipalBindingInstallation(denial)
        }
        Query::PrincipalIdentityEncoding(denial) => {
            BankEstateProgressionDenial::PrincipalIdentityEncoding(denial)
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
        Query::Authorization(denial) => BankEstateProgressionDenial::from_authorization(denial),
        Query::Projection(denial) => BankEstateProgressionDenial::from_projection(denial),
        Query::Decision(denial) => BankEstateProgressionDenial::from_decision_projection(denial),
        Query::Attempt(denial) => BankEstateProgressionDenial::from_attempt(denial),
    }
}
