mod projection;

pub(crate) use projection::project_release_readiness;
pub use projection::BankEstateReleaseProjectionDenial;

use bank_domain::{
    estate::{EstateAction, EstateCaseId, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
};

use super::BankEstateProgressionDenial;

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
