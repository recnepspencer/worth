use super::*;
pub enum SettlementReceiptCustody {
    Owned(WorthQueryApplicationCommitReceipt),
    Recovered(super::super::WorthQueryRecoveredProducerPublication),
    Ready(ReadyCompletion),
}
impl SettlementReceiptCustody {
    pub(super) fn as_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        match self {
            Self::Owned(receipt) => Some(receipt),
            Self::Recovered(_) => None,
            Self::Ready(completion) => match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => Some(receipt),
                WorthQueryAcceptedOutputAuthority::Recovered(_)
                | WorthQueryAcceptedOutputAuthority::Stable(_)
                | WorthQueryAcceptedOutputAuthority::Restored(_) => None,
            },
        }
    }
    pub(super) fn publication_view(&self) -> Option<super::super::OutputPublicationView<'_>> {
        match self {
            Self::Owned(receipt) => Some(receipt.into()),
            Self::Recovered(publication) => Some(publication.view()),
            Self::Ready(completion) => completion.authority.publication_view(),
        }
    }
    pub(super) fn recovered(
        &self,
    ) -> Option<&super::super::WorthQueryRecoveredProducerPublication> {
        match self {
            Self::Recovered(publication) => Some(publication),
            Self::Owned(_) => None,
            Self::Ready(completion) => match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Recovered(publication) => Some(publication),
                WorthQueryAcceptedOutputAuthority::Committed(_)
                | WorthQueryAcceptedOutputAuthority::Stable(_)
                | WorthQueryAcceptedOutputAuthority::Restored(_) => None,
            },
        }
    }
}
