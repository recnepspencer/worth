//! Crash-consistency oracle for one fuzzy Store physical checkpoint above
//! committed journal entries.
//!
//! The oracle counts every media operation of every role the checkpoint
//! performs, then reruns it once per (role, ordinal, directive) with a fault
//! at exactly that operation, abandons the runtime and recovers through the
//! production path. Recovery must settle whatever the interrupted checkpoint
//! left: the recovered runtime serves without inspection, holds every entry,
//! publishes the next one, and a second recovery reaches the same journal.
//! For the faults that leave obligations recovery must adjudicate, the first
//! recovery is also killed just before it discharges them, and the next one
//! must settle to the same result.

#[allow(unused_imports)] use crate::production_entry::physical_checkpoint_oracle::*;
use {std::num::NonZeroU64};
use {std::path::Path};

use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaOperationRole};
use {worth_store::physical_runtime::OrderedJournalPublication, worth_store::physical_runtime::PhysicalRecoveryYieldpointStage, worth_store::physical_runtime::ServingPhysicalRuntime};

use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal::recovery_interruption::kill_recovery_at};
use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_proof::TransitionOutcome};

#[test]
fn every_media_operation_of_a_checkpoint_replacement_recovers() {
    sweep(crate::production_entry::physical_checkpoint_oracle::Target::Replacement);
}

#[test]
fn every_media_operation_of_a_first_checkpoint_recovers() {
    sweep(crate::production_entry::physical_checkpoint_oracle::Target::First);
}
