//! Exact retained provenance disagreement; diagnostic data grants no authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputSettlementProvenanceDenial {
    OriginalSettlementMissing,
    ReceiptSettlementMissing,
    ForeignSettlementOwner,
    OriginalDisclosureMissing,
    ReceiptDisclosureMissing,
    ForeignDisclosureOwner,
    PublicationMismatch,
    ReusedSettlementOwnerMismatch,
    RestoredCheckpointMismatch,
}
