use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        BankPrincipalBinding, BankSchema, CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::WorthQueryApplicationMandatoryReviewDenial,
    declaration::application_schema::TypedMutationPreconditions,
};

use super::{
    lifecycle_facts::seal_review_lifecycle_facts, BankEstateLifecycleProjectionDenial,
    BankEstateMandatoryReview, BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial,
    BankEstateProgressionFailure,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn complete_estate_mandatory_review_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        mandatory: BankEstateMandatoryReview,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateMandatoryReviewOutcome,
        BankEstateProgressionFailure<BankEstateMandatoryReview>,
    > {
        let EstateAction::CompleteMandatoryReview { access, review, .. } = action else {
            return Err(BankEstateProgressionFailure::retained(
                BankEstateProgressionDenial::CommandInput("CompleteEstateMandatoryReviewOperation"),
                mandatory,
            ));
        };
        let outcome = self
            .request(principal, request)
            .execute_mandatory_review_in_program(
                self.application_program(),
                BankPrincipalBinding::reference(),
                CompleteEstateMandatoryReviewCapability::reference(),
                CompleteEstateMandatoryReviewOperation::reference(),
                mandatory.into_query(),
                action,
                idempotency_key,
                TypedMutationPreconditions::<
                    BankSchema,
                    CompleteEstateMandatoryReviewOperation,
                    bank_domain::schema::EstateCase,
                >::default(),
                self.invariant_projection(),
                |reader, estate| seal_review_lifecycle_facts(reader, access, review, estate),
            );
        match outcome {
            Ok(outcome) => Ok(BankEstateMandatoryReviewOutcome::from_query(outcome)),
            Err(failure) => {
                let (denial, mandatory) = failure.into_parts();
                let denial = map_review_denial(denial);
                Err(match mandatory {
                    Some(mandatory) => BankEstateProgressionFailure::retained(
                        denial,
                        BankEstateMandatoryReview::from_query(mandatory),
                    ),
                    None => BankEstateProgressionFailure::consumed(denial),
                })
            }
        }
    }
}

fn map_review_denial(
    denial: WorthQueryApplicationMandatoryReviewDenial<BankEstateLifecycleProjectionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationMandatoryReviewDenial as Query;
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
        Query::ReviewAuthorization(denial) => BankEstateProgressionDenial::ReviewAuthorization(
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
