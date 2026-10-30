//! Only positively owner-free preparation failures can relinquish the acceptance slot.
use super::*;

impl PreparationFailure<'_> {
    pub(in crate::proposal_acceptance::application) fn try_discard_uncreated(
        self,
    ) -> Result<(), Self> {
        let owner_free = match &self {
            Self::Before(cause) => owner_free_refusal(cause),
            Self::Refused { cause, handoff } => {
                owner_free_refusal(cause) && handoff.as_ref().is_none_or(owner_free_advancement)
            }
            Self::Advancement(cause) => owner_free_advancement(cause),
            Self::Created { .. } => false,
        };
        if owner_free {
            Ok(())
        } else {
            Err(self)
        }
    }
}

fn owner_free_refusal(cause: &PreparationRefusal) -> bool {
    match cause {
        PreparationRefusal::Request(cause) => owner_free_request(cause),
        PreparationRefusal::Creation(cause) => match cause {
            WorthQueryProductBranchCreateError::ComponentsIncomplete
            | WorthQueryProductBranchCreateError::IdentityExhausted
            | WorthQueryProductBranchCreateError::SourceAdmission(_)
            | WorthQueryProductBranchCreateError::Creation(_) => true,
            WorthQueryProductBranchCreateError::ProductUnpublished(_) => false,
        },
        PreparationRefusal::UnsupportedVersion(_)
        | PreparationRefusal::EmptyProposal
        | PreparationRefusal::ModelMismatch
        | PreparationRefusal::ModelHeadMismatch
        | PreparationRefusal::TargetMismatch
        | PreparationRefusal::Mutation(_)
        | PreparationRefusal::Description(_)
        | PreparationRefusal::Ordinal(_)
        | PreparationRefusal::Reservation(_)
        | PreparationRefusal::Selection(_)
        | PreparationRefusal::Fork(_) => true,
    }
}

fn owner_free_request(cause: &Failure) -> bool {
    match cause {
        Failure::Advancement(cause) => owner_free_advancement(cause),
        Failure::Creation(_)
        | Failure::Close(_)
        | Failure::ModelCreation(_)
        | Failure::Recording(_) => false,
        Failure::Authentication(_)
        | Failure::Selection(_)
        | Failure::Query(_)
        | Failure::Mutation(_)
        | Failure::Encoding(_)
        | Failure::MissingModel
        | Failure::MissingTarget
        | Failure::ModelMismatch
        | Failure::StorageMismatch
        | Failure::ProposalUnavailable
        | Failure::AmbiguousProposal
        | Failure::ProposalOccurrenceChanged
        | Failure::ProposalRevisionChanged => true,
    }
}

fn owner_free_advancement(cause: &WorthQueryAdvancementDenial) -> bool {
    match cause {
        WorthQueryAdvancementDenial::Resource(_)
        | WorthQueryAdvancementDenial::Interrupted(_)
        | WorthQueryAdvancementDenial::NestedOpening
        | WorthQueryAdvancementDenial::NestedStopped => true,
        WorthQueryAdvancementDenial::Panicked => false,
    }
}
