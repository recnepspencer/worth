//! Explicit payload-copy and structural comparison work, separate from byte custody.
use super::LineageRecord;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};

impl LineageRecord {
    pub(crate) fn admit_copy_work(&self, work: &mut Work) -> Result<(), Denial> {
        let bytes = Charge::capacity::<Self>(1)?
            .checked_add(self.retained_heap_charge(work)?)?
            .bytes();
        work.reserve_visits(usize::try_from(bytes).map_err(|_| Denial::ChargeOverflow)?)
    }

    pub(crate) fn admit_comparison_work(
        &self,
        other: &Self,
        work: &mut Work,
    ) -> Result<(), Denial> {
        // Every nested String/vector comparison is bounded by these concrete
        // retained extents. Measurement itself admits each structural visit.
        self.admit_copy_work(work)?;
        other.admit_copy_work(work)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SignalBranchId;
    #[test]
    fn nested_names_require_copy_and_comparison_work_before_use() {
        let record = LineageRecord::branch_fork(
            0,
            SignalBranchId(0),
            SignalBranchId(1),
            SignalBranchId(0),
            "child".repeat(4096),
            "parent".repeat(4096),
        );
        let mut measured = Work::new(1_000_000);
        record.admit_copy_work(&mut measured).unwrap();
        let exact = measured.visits();
        assert!(record.admit_copy_work(&mut Work::new(exact - 1)).is_err());
        record.admit_copy_work(&mut Work::new(exact)).unwrap();
        assert!(record
            .admit_comparison_work(&record, &mut Work::new(exact))
            .is_err());
        record
            .admit_comparison_work(&record, &mut Work::new(exact * 2))
            .unwrap();
    }
}
