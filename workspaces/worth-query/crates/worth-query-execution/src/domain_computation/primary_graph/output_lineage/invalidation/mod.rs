//! Derived consumed-fact postings and source-position-specific output marks.

#[cfg(test)]
mod canonical_binding_order;
mod commit_touches;
mod computation_changes;
pub(in crate::domain_computation::primary_graph) use computation_changes::ComputationChanges;
pub(in crate::domain_computation::primary_graph) use computation_changes::ComputationRegistration;
pub(in crate::domain_computation::primary_graph) use computation_changes::PostconditionCarry;

mod delivery;
mod delivery_observation;
pub use delivery_observation::inexact_native_deliveries_on_this_thread_for_test;
mod derived;

mod equality;
mod equality_recovery;
pub(in crate::domain_computation::primary_graph) use equality_recovery::EqualityRecoveryRow;
mod fact_key;
mod fact_keys;

mod full_verification;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationDecision;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationImage;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationStop;
mod head_cell;
pub(in crate::domain_computation) use head_cell::HeadCellRegistrationStop;

mod equivalence_control;
mod logical_marking;
mod marking_observation;
pub use logical_marking::WorthQueryNativeMarkingPrecision;
pub use logical_marking::WorthQueryNativeMarkingWork;
pub use marking_observation::native_marking_on_this_thread_for_test;
pub use marking_observation::WorthQueryMarkedOutputForTest;
pub use marking_observation::WorthQueryNativeMarkingForTest;
mod mark_state;
mod output_availability;
pub(in crate::domain_computation::primary_graph) use output_availability::SourcePublicationReadFence;
mod output_facts;
pub(in crate::domain_computation) use output_availability::ObservedSourcePublication;
pub(in crate::domain_computation::primary_graph) use output_facts::RegisteredOutputFacts;

mod owner;
mod publication;
mod publication_cell;
mod query_coverage;
mod read_basis_carry;
pub(in crate::domain_computation::primary_graph) use query_coverage::QueryCoverageStop;
pub(in crate::domain_computation::primary_graph) use query_coverage::QueryFactCoverage;
mod reestablishment;

mod retirement;
mod settlement;
mod source_alignment;
mod source_publication;
mod touch_keys;
mod verification;
mod verified_current;

#[cfg(test)]
mod native_journey_tests;

pub(in crate::domain_computation::primary_graph) use commit_touches::CommitTouchEntities;
pub(in crate::domain_computation::primary_graph) use commit_touches::CommitTouchInterest;
pub(in crate::domain_computation::primary_graph) use commit_touches::TouchedCommits;

pub(in crate::domain_computation::primary_graph) use derived::ConsumedOutputCurrentness;
pub(in crate::domain_computation::primary_graph) use derived::CurrentSettlementRegistrationCleanup;
pub(in crate::domain_computation::primary_graph) use derived::PreparedCurrentSettlementRegistration;
pub(in crate::domain_computation::primary_graph) use derived::SettlementRegistrationStop;
pub(in crate::domain_computation::primary_graph) use derived::SourceSettlementCurrentness;

pub(in crate::domain_computation::primary_graph) use mark_state::FullVerificationReason;
pub(in crate::domain_computation) use owner::SourceInvalidationOwner;
pub(in crate::domain_computation::primary_graph) use publication::collect_consumed_output_upstream;
pub(in crate::domain_computation::primary_graph) use publication::register_completed;
pub(in crate::domain_computation::primary_graph) use publication::register_republished;
pub(in crate::domain_computation::primary_graph) use settlement::PostingContinuation;
pub(in crate::domain_computation::primary_graph) use settlement::PostingRebaseBasis;
pub(in crate::domain_computation::primary_graph) use settlement::SettlementRegistration;
pub(in crate::domain_computation::primary_graph) use verification::DirtyReverification;
pub(in crate::domain_computation::primary_graph) use verification::SettlementVerificationStop;
pub(in crate::domain_computation::primary_graph) use verified_current::VerifiedCurrentCleanup;
