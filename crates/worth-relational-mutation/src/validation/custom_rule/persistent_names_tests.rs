//! An invariant's name reads are charged to its work meter: once the budget
//! is spent a read refuses `WorkExhausted` and never reaches the names.
use std::cell::Cell;
use std::num::NonZeroU64;

use super::{
    CustomInvariantNamePosture, CustomInvariantNameReadDenial, CustomInvariantNameSource,
    CustomInvariantNameView, CustomInvariantRecordName, InvariantNames,
};
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::{
    LineageRecordTarget, PersistentNameBirthParts, PersistentNameSubjectKind,
};
use crate::validation::custom_rule::CustomInvariantWorkMeter;

struct NoNames;
impl CustomInvariantNameView for NoNames {
    fn record_name(&self, _: LineageRecordTarget) -> CustomInvariantRecordName {
        CustomInvariantRecordName::Unnamed
    }
    fn recorded_name(
        &self,
        _: PersistentNameSubjectKind,
        _: &[u8; 32],
    ) -> Option<PersistentNameKey> {
        None
    }
    fn scheme_of(&self, _: PersistentNameKey) -> Option<&str> {
        None
    }
    fn birth_of(&self, _: PersistentNameKey) -> Option<&PersistentNameBirthParts> {
        None
    }
    fn born_record(&self, _: PersistentNameKey) -> Option<LineageRecordTarget> {
        None
    }
}

/// Counts the reads that reached it.
#[derive(Default)]
struct Counted(std::sync::atomic::AtomicUsize);
impl CustomInvariantNameSource for Counted {
    fn read(
        &self,
        _: CustomInvariantNamePosture,
        read: &mut dyn FnMut(&dyn CustomInvariantNameView),
    ) -> Result<(), CustomInvariantNameReadDenial> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        read(&NoNames);
        Ok(())
    }
    fn scheme_subject(&self, _: &str) -> Option<(PersistentNameSubjectKind, String)> {
        None
    }
}

#[test]
fn a_spent_work_budget_refuses_name_reads_with_work_exhausted() {
    let source = Counted::default();
    let meter = CustomInvariantWorkMeter::new(NonZeroU64::new(1));
    let names = InvariantNames::new(Some(&source), meter);
    let reads = names.posture(CustomInvariantNamePosture::Proposed);
    let answered = Cell::new(0);
    let read = || reads.read(|_| answered.set(answered.get() + 1));
    assert_eq!(read(), Ok(()), "the first read fits the budget");
    assert_eq!(read(), Err(CustomInvariantNameReadDenial::WorkExhausted));
    assert_eq!(answered.get(), 1);
    assert_eq!(source.0.load(std::sync::atomic::Ordering::Relaxed), 1);
}

#[test]
fn a_validation_without_names_refuses_unavailable() {
    let names = InvariantNames::new(None, CustomInvariantWorkMeter::new(None));
    let reads = names.posture(CustomInvariantNamePosture::Committed);
    assert_eq!(
        reads.read(|_| ()),
        Err(CustomInvariantNameReadDenial::Unavailable)
    );
}
