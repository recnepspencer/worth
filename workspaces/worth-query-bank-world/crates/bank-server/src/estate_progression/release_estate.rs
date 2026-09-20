mod projection;

pub(crate) use projection::project_release_readiness;
pub use projection::BankEstateReleaseProjectionDenial;

use bank_domain::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use super::BankEstateProgressionDenial;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankMutationCommitOutcome};

impl BankIdentityRuntime {
    pub fn release_estate_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
        let EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } = action
        else {
            return Err(BankEstateProgressionDenial::CommandInput(
                "ReleaseEstateOperation",
            ));
        };
        super::program_outcome::program_outcome(
            self.request(principal, request)
                .mutate(bank_domain::schema::ReleaseEstate::new(
                    estate, executor, authority, review,
                ))
                .idempotency(key)
                .execute_capability_in_selected_program(self.application_program()),
            "ReleaseEstateOperation",
            |denial| {
                denial
                    .downcast::<BankEstateReleaseProjectionDenial>()
                    .map(BankEstateProgressionDenial::EstateReleaseProjection)
            },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ReleaseCommand {
    pub(crate) estate: EstateCaseId,
    pub(crate) executor: BankPrincipalId,
    pub(crate) authority: LegalAuthorityId,
    pub(crate) review: MandatoryReviewId,
}

pub(crate) fn release_command(
    action: EstateAction,
) -> Result<ReleaseCommand, BankEstateProgressionDenial> {
    match action {
        EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } => Ok(ReleaseCommand {
            estate,
            executor,
            authority,
            review,
        }),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "ReleaseEstateOperation",
        )),
    }
}
