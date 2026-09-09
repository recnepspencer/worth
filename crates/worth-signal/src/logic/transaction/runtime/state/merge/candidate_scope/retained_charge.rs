use super::ScopedMergeCandidateBreadthSummary;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl RetainedStorageMeasurement for ScopedMergeCandidateBreadthSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            boundary_candidate_width: _,
            requested_scope_width: _,
            admitted_candidate_width: _,
            skipped_scope_width: _,
            no_op_scope_width: _,
            support_closure_width: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
