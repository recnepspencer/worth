//! Derived consumed-fact postings and source-position-specific output marks.

mod admission;
#[cfg(test)]
mod canonical_binding_order;
mod commit_touches;
mod computation_changes;
pub(in crate::domain_computation::primary_graph) use computation_changes::PostconditionCarry;
pub(in crate::domain_computation::primary_graph) use computation_changes::{
    ComputationChanges, ComputationRegistration,
};
mod consumed_capacity;
mod consumed_capacity_observation;
mod delivery;
mod delivery_observation;
pub use delivery_observation::inexact_native_deliveries_on_this_thread_for_test;
mod derived;
mod edit_admission;
mod equality;
mod equality_recovery;
pub(in crate::domain_computation::primary_graph) use equality_recovery::EqualityRecoveryRow;
mod fact_key;
mod fact_keys;
mod fact_retention;
mod full_verification;
pub(in crate::domain_computation::primary_graph) use full_verification::{
    FullVerificationDecision, FullVerificationImage, FullVerificationStop,
};
mod head_cell;
pub(in crate::domain_computation) use head_cell::HeadCellRegistrationStop;
mod index_capacity;
pub(in crate::domain_computation::primary_graph) use index_capacity::{
    arc_bytes, insertion_sequence_bytes, ordered_navigation_work,
};
mod equivalence_control;
mod logical_marking;
mod marking_observation;
pub use logical_marking::{WorthQueryNativeMarkingPrecision, WorthQueryNativeMarkingWork};
pub use marking_observation::{
    native_marking_on_this_thread_for_test, WorthQueryMarkedOutputForTest,
    WorthQueryNativeMarkingForTest,
};
mod mark_state;
mod output_availability;
pub(in crate::domain_computation::primary_graph) use output_availability::SourcePublicationReadFence;
mod output_facts;
pub(in crate::domain_computation) use output_availability::ObservedSourcePublication;
pub(in crate::domain_computation::primary_graph) use output_facts::RegisteredOutputFacts;
mod output_witness_capacity;
mod owner;
mod publication;
mod publication_cell;
mod query_coverage;
mod read_basis_carry;
pub(in crate::domain_computation::primary_graph) use query_coverage::{
    QueryCoverageStop, QueryFactCoverage,
};
mod reestablishment;
mod retention;
mod retirement;
mod settlement;
mod source_alignment;
mod source_publication;
mod touch_keys;
mod verification;
mod verified_current;

#[cfg(test)]
mod native_journey_tests;

pub(in crate::domain_computation::primary_graph) use commit_touches::{
    CommitTouchEntities, CommitTouchInterest, CommitTouchesStop, TouchedCommits,
};
pub(in crate::domain_computation::primary_graph) use consumed_capacity::RetainedConsumedOutputCapacity;
pub(in crate::domain_computation::primary_graph) use derived::{
    ConsumedOutputCurrentness, CurrentSettlementRegistrationCleanup,
    PreparedCurrentSettlementRegistration, SettlementRegistrationStop, SourceSettlementCurrentness,
};
pub(in crate::domain_computation) use edit_admission::InvalidationEditAdmission;
pub(in crate::domain_computation::primary_graph) use edit_admission::{
    CarriedRequestInvalidationAdmission, ReservedExternalWork,
};
pub(in crate::domain_computation::primary_graph) use fact_retention::arc_slice_bytes;

pub(in crate::domain_computation::primary_graph) fn retained_fact_payload_bytes(
    fact: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    admission: &mut InvalidationEditAdmission,
) -> Result<u64, worth_relational::facade::mvcc::CompanionPreflightStop> {
    fact_retention::fact_payload_bytes(fact, admission)
}
pub(in crate::domain_computation::primary_graph) use mark_state::FullVerificationReason;
pub(in crate::domain_computation) use owner::SourceInvalidationOwner;
pub(in crate::domain_computation::primary_graph) use publication::collect_consumed_output_upstream;
pub(in crate::domain_computation::primary_graph) use publication::register_completed;
pub(in crate::domain_computation::primary_graph) use publication::register_republished;
pub(in crate::domain_computation::primary_graph) use settlement::{
    PostingContinuation, PostingRebaseBasis, SettlementRegistration,
};
pub(in crate::domain_computation::primary_graph) use verification::{
    DirtyReverification, SettlementVerificationStop,
};
pub(in crate::domain_computation::primary_graph) use verified_current::VerifiedCurrentCleanup;

pub use read_basis_carry::carried_read_basis_custody_for_test;
