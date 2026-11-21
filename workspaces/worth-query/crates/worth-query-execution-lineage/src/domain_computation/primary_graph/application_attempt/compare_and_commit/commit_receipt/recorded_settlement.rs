//! Descriptive settlement coordinates never grant output or delivery authority.
use super::WorthQueryApplicationCommitReceipt;
use worth_foundational::facade::DurableQueryOutputSettlement;
impl WorthQueryApplicationCommitReceipt {
    /// The exact occurrence-local slot issued by the output owner, paired with
    /// this receipt's actual World publication. No output grant is reconstructed.
    pub fn recorded_output_settlement(&self) -> DurableQueryOutputSettlement {
        match self.exact_output_settlement.as_ref() {
            None => DurableQueryOutputSettlement::Absent,
            Some(settlement) => DurableQueryOutputSettlement::OriginalOccurrenceSlot {
                slot: u64::try_from(settlement.address().2)
                    .expect("a platform output slot fits the recorded u64 address"),
            },
        }
    }
}
