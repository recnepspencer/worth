//! Validate immutable membership before applying original lifecycle progress.
use super::super::{QueryObligationDenial as D, QueryObligationLedger};
use super::*;
impl QueryObligationLedger {
    pub fn original_operations(&self) -> impl Iterator<Item = &QueryOriginalOperationObligation> {
        self.original_operations.values()
    }
    pub(in crate::query_obligations) fn validate_original_operation(
        &self,
        record: &QueryOriginalOperationRecord,
        position: u64,
    ) -> Result<(), D> {
        let identity = match record {
            QueryOriginalOperationRecord::Retain(row) => row.identity,
            QueryOriginalOperationRecord::Advance { identity, .. }
            | QueryOriginalOperationRecord::Discharge { identity } => *identity,
        };
        if identity.home() != self.home() {
            return Err(D::ForeignHome);
        }
        match record {
            QueryOriginalOperationRecord::Retain(row) => {
                if identity.acquire_position().get() != position
                    || self.original_operations.contains_key(&identity)
                {
                    return Err(D::ObligationSubjectMismatch);
                }
                if row.data.program.is_empty()
                    || row.data.action.is_empty()
                    || row.data.binding.is_empty()
                    || row.data.branch.is_empty()
                    || row.data.description.is_empty()
                    || row.position != QueryOriginalOperationPosition::Initial
                {
                    return Err(D::MalformedFrame);
                }
                if self.status(&worth_foundational::facade::CanonicalDigestId::new(
                    row.data.revision,
                )) != Some(worth_foundational::facade::ProgramSupportStatus::Active)
                {
                    return Err(D::SupportUnavailable);
                }
            }
            QueryOriginalOperationRecord::Advance { .. } => {
                self.original_operations
                    .get(&identity)
                    .ok_or(D::UnknownObligation)?;
            }
            QueryOriginalOperationRecord::Discharge { .. } => {
                self.original_operations
                    .get(&identity)
                    .ok_or(D::UnknownObligation)?;
            }
        }
        Ok(())
    }
    pub(in crate::query_obligations) fn apply_original_operation(
        &mut self,
        record: QueryOriginalOperationRecord,
    ) {
        match record {
            QueryOriginalOperationRecord::Retain(row) => {
                self.original_operations.insert(row.identity, row);
            }
            QueryOriginalOperationRecord::Advance { identity, position } => {
                self.original_operations
                    .get_mut(&identity)
                    .expect("validated original")
                    .position = position
            }
            QueryOriginalOperationRecord::Discharge { identity } => {
                self.original_operations.remove(&identity);
            }
        }
    }
}
