//! Actual compiled projection operations summarized for Source READ admission.
use super::occurrences::MaterialDenial;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Stop;
#[derive(Default)]
pub struct SourceProjectionWork {
    pub projected_records: usize,
    pub projected_fields: usize,
    pub relation_records_examined: usize,
}
impl SourceProjectionWork {
    pub fn append(&mut self, other: Self) -> Result<(), MaterialDenial> {
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
    pub fn complete(
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
