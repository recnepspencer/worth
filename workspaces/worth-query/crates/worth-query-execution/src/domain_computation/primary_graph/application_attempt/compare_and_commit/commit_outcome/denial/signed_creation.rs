use {super::WorthQueryApplicationCommitDenial, crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind, super::WorthQueryApplicationCommitDenialStage};
impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn signed_creation_authentication(
        kind: worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SignedCreationAuthentication { kind },
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn signed_entity_immutable(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::SignedEntityImmutable,
            stage: WorthQueryApplicationCommitDenialStage::ProposalBinding,
            detail: None,
            cause: None,
        }
    }
}
