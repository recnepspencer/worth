use super::*;

pub(super) enum SettlementReceiptCustody {
    Owned(WorthQueryApplicationCommitReceipt),
    Ready(ReadyCompletion),
}

impl SettlementReceiptCustody {
    pub(super) fn as_receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        match self {
            Self::Owned(receipt) => receipt,
            Self::Ready(completion) => match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => receipt,
                _ => unreachable!("only committed Ready authority carries a receipt"),
            },
        }
    }
}
