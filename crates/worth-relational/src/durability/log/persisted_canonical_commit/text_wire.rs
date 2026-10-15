//! Execution-taking input for the current native canonical entry.
use serde::Deserialize;
#[derive(Deserialize)]
pub(crate) struct CanonicalCommitWire<'a> {
    canonical_protocol_version: u16,
    position: crate::publication::patch::data::PatchStreamPosition,
    #[serde(borrow, rename = "canonical_v4")]
    canonical: crate::history::data::CanonicalEnvelopeWire<'a>,
}
impl CanonicalCommitWire<'_> {
    pub(crate) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::PersistedCanonicalCommit, crate::durability::data::RelationalRecoveryStop>
    {
        if self.canonical_protocol_version != 4 {
            return Err(crate::durability::data::DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CorruptSegment,
                "unsupported native canonical protocol; managed text requires v4",
            )
            .into());
        }
        let canonical = self
            .canonical
            .readmit(execution)
            .map_err(crate::symbols::data::field_text_recovery_stop)?;
        Ok(super::PersistedCanonicalCommit {
            canonical_protocol_version: 4,
            position: self.position,
            canonical,
        })
    }
}
