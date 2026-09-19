use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        BankPrincipalBinding, BankSchema,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::WorthQueryApplicationElevationApprovalDenial,
    declaration::application_schema::TypedMutationPreconditions,
};

use super::{
    lifecycle_facts::{approval_lifecycle_identities, seal_approval_lifecycle_facts},
    BankEstateElevationApprovalOutcome, BankEstateLifecycleProjectionDenial,
    BankEstateProgressionDenial, BankEstateProgressionFailure, BankRequestedEstateElevation,
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
        let (access_identity, review_identity) =
            match approval_lifecycle_identities(requested.query()) {
                Ok(identities) => identities,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(
                        BankEstateProgressionDenial::LifecycleProjection(denial),
                        requested,
                    ))
                }
            };
        let outcome = self
            .request(principal, request)
            .execute_elevation_approval_in_program(
                self.application_program(),
                BankPrincipalBinding::reference(),
                ApproveEstateEmergencyAccessCapability::reference(),
                ApproveEstateEmergencyAccessOperation::reference(),
                requested.into_query(),
                action,
                idempotency_key,
                TypedMutationPreconditions::<
                    BankSchema,
                    ApproveEstateEmergencyAccessOperation,
                    bank_domain::schema::EstateCase,
                >::default(),
                self.invariant_projection(),
                |reader, estate| {
                    seal_approval_lifecycle_facts(reader, access_identity, review_identity, estate)
                },
            );
        match outcome {
            Ok(outcome) => Ok(BankEstateElevationApprovalOutcome::from_query(outcome)),
            Err(failure) => {
                let (denial, requested) = failure.into_parts();
                let denial = map_approval_denial(denial);
                Err(match requested {
                    Some(requested) => BankEstateProgressionFailure::retained(
                        denial,
                        BankRequestedEstateElevation::from_query(requested),
                    ),
                    None => BankEstateProgressionFailure::consumed(denial),
                })
            }
        }
    }
}

fn map_approval_denial(
    denial: WorthQueryApplicationElevationApprovalDenial<BankEstateLifecycleProjectionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationElevationApprovalDenial as Query;
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
        Query::ApprovalAuthorization(denial) => BankEstateProgressionDenial::ApprovalAuthorization(
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
