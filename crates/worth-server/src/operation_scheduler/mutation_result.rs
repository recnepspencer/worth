use worth_query::facade::runtime::{
    WorthQueryBatchWriteReceipt, WorthQueryBatchWriteReceiptInspection, WorthQueryWriteReceipt,
    WorthQueryWriteReceiptInspection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthServerScheduledMutationResult {
    Single {
        receipt: Box<WorthQueryWriteReceipt>,
        inspection: Box<WorthQueryWriteReceiptInspection>,
    },
    Batch {
        receipt: Box<WorthQueryBatchWriteReceipt>,
        inspection: Box<WorthQueryBatchWriteReceiptInspection>,
    },
}

impl WorthServerScheduledMutationResult {
    pub fn result_digest(
        &self,
    ) -> Result<&str, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>
    {
        match self {
            Self::Single { receipt, .. } => Ok(receipt
                .commit_evidence_identity()
                .terminal_projection_for_reporting()),
            Self::Batch { receipt, .. } => receipt.batch_digest(),
        }
    }

    pub fn snapshot_basis_digest(&self) -> String {
        match self {
            Self::Single { receipt, .. } => receipt
                .snapshot_identity()
                .terminal_projection_for_reporting()
                .to_string(),
            Self::Batch { receipt, .. } => receipt
                .write_receipts()
                .last()
                .expect("scheduled batch mutation should retain at least one write receipt")
                .snapshot_identity()
                .terminal_projection_for_reporting()
                .to_string(),
        }
    }
}
