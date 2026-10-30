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

#[allow(unused_imports)]
use crate::production_entry::physical_checkpoint_oracle::*;

#[test]
fn every_media_operation_of_a_checkpoint_replacement_recovers() {
    sweep(crate::production_entry::physical_checkpoint_oracle::Target::Replacement);
}

#[test]
fn every_media_operation_of_a_first_checkpoint_recovers() {
    sweep(crate::production_entry::physical_checkpoint_oracle::Target::First);
}
