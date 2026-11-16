//! Actual compiled projection operations summarized for Source READ admission.
use super::occurrences::MaterialDenial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Stop;
#[derive(Default)]
pub(super) struct SourceProjectionWork {
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) relation_records_examined: usize,
}
impl SourceProjectionWork {
    pub(super) fn append(&mut self, other: Self) -> Result<(), MaterialDenial> {
        self.projected_records = self
            .projected_records
            .checked_add(other.projected_records)
            .ok_or(Stop::ChargedBytesOverflow)?;
        self.projected_fields = self
            .projected_fields
            .checked_add(other.projected_fields)
            .ok_or(Stop::ChargedBytesOverflow)?;
        self.relation_records_examined = self
            .relation_records_examined
            .checked_add(other.relation_records_examined)
            .ok_or(Stop::ChargedBytesOverflow)?;
        Ok(())
    }
    pub(super) fn complete(
        self,
        examined_candidates: usize,
    ) -> crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
        crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
            examined_candidates,
            projected_records: self.projected_records,
            projected_fields: self.projected_fields,
            relation_records_examined: self.relation_records_examined,
            // Physical Source membership is canonical; no user ordering runs.
            ordering_comparisons: 0,
        }
    }
}
