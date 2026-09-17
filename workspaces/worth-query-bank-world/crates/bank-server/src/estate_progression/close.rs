use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        BankPrincipalBinding, BankSchema, RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::WorthQueryApplicationElevationCloseDenial,
    declaration::application_schema::TypedMutationPreconditions,
};

use super::{
    lifecycle_facts::seal_close_lifecycle_facts, BankApprovedEstateElevation,
    BankEstateElevationCloseOutcome, BankEstateLifecycleProjectionDenial,
    BankEstateProgressionDenial, BankEstateProgressionFailure,
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
        let EstateAction::RevokeEmergencyAccess { access, .. } = action else {
            return Err(BankEstateProgressionFailure::retained(
                BankEstateProgressionDenial::CommandInput("RevokeEstateEmergencyAccessOperation"),
                approved,
            ));
        };
        let outcome = self
            .request(principal, request)
            .execute_elevation_close_in_program(
                self.application_program(),
                BankPrincipalBinding::reference(),
                RevokeEstateEmergencyAccessCapability::reference(),
                RevokeEstateEmergencyAccessOperation::reference(),
                approved.into_query(),
                action,
                idempotency_key,
                TypedMutationPreconditions::<
                    BankSchema,
                    RevokeEstateEmergencyAccessOperation,
                    bank_domain::schema::EstateCase,
                >::default(),
                self.invariant_projection(),
                |reader, estate| seal_close_lifecycle_facts(reader, access, estate),
            );
        match outcome {
            Ok(outcome) => Ok(BankEstateElevationCloseOutcome::from_query(outcome)),
            Err(failure) => {
                let (denial, approved) = failure.into_parts();
                let denial = map_close_denial(denial);
                Err(match approved {
                    Some(approved) => BankEstateProgressionFailure::retained(
                        denial,
                        BankApprovedEstateElevation::from_query(approved),
                    ),
                    None => BankEstateProgressionFailure::consumed(denial),
                })
            }
        }
    }
}

fn map_close_denial(
    denial: WorthQueryApplicationElevationCloseDenial<BankEstateLifecycleProjectionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationElevationCloseDenial as Query;
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
        Query::CloseAuthorization(denial) => BankEstateProgressionDenial::CloseAuthorization(
            crate::BankAuthorizationDenial::from_query(denial),
        ),
        Query::Projection(denial) => BankEstateProgressionDenial::from_projection(denial),
        Query::Decision(denial) => BankEstateProgressionDenial::LifecycleProjection(denial),
        Query::Attempt(denial) => BankEstateProgressionDenial::from_attempt(denial),
        Query::IdempotencyResolution(denial) => {
            BankEstateProgressionDenial::from_idempotency(denial)
        }
    }
}
