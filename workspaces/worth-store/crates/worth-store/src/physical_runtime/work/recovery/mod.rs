mod effect_classification;
mod effect_inventory;
mod effect_obligation;
mod format_mapping;
mod integrity_admission;
mod journal_counters;
mod locator;
mod observation;
mod recovered_admission;

#[cfg(test)]
mod effect_classification_tests;
#[cfg(test)]
mod effect_obligation_tests;
#[cfg(test)]
mod format_mapping_tests;

pub(in crate::physical_runtime) use effect_inventory::PhysicalEffectRecoveryInventory;
pub(in crate::physical_runtime) use effect_obligation::{
    PhysicalEffectJournal, PreparedPhysicalEffect,
};
pub use journal_counters::PhysicalRecoveryJournalCounters;
pub use locator::{
    PhysicalCheckpointRecoveryAction, PhysicalWorkRecoveryLocator, PhysicalWorkRecoveryTarget,
};
pub use observation::{
    PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryAdmissionObservation,
    PhysicalWorkRecoveryAdmissionOutcome, PhysicalWorkRecoveryIngressRejection,
    PhysicalWorkRecoveryObservationSubject,
};
pub(in crate::physical_runtime) use recovered_admission::admit_recovered_obligation;
