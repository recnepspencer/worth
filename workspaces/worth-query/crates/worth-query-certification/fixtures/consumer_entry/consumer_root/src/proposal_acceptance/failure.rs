//! Ordinary owner denials survive application-level recording and observation.
use super::{creation, recording};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAuthenticationDenial,
    application_contribution::WorthQueryAdvancementDenial, application_entry::*, product::*,
};

#[derive(Debug)]
pub enum Failure {
    Authentication(WorthQueryAuthenticationDenial),
    Advancement(WorthQueryAdvancementDenial),
    Creation(WorthQueryProductBranchCreateError),
    Selection(WorthQueryProductBranchAdmissionDenial),
    Query(WorthQueryApplicationRequestQueryDenial),
    Mutation(WorthQueryApplicationRequestMutationDenial),
    ModelCreation(WorthQueryApplicationMutationOutcome<creation::Denial, creation::Created>),
    Recording(WorthQueryApplicationMutationOutcome<recording::Denial, recording::Recorded>),
    Close(WorthQueryApplicationProductBranchCloseDenial),
    Encoding(serde_json::Error),
    MissingModel,
    MissingTarget,
    ModelMismatch,
    StorageMismatch,
    ProposalUnavailable,
    AmbiguousProposal,
    ProposalOccurrenceChanged,
    ProposalRevisionChanged,
}
