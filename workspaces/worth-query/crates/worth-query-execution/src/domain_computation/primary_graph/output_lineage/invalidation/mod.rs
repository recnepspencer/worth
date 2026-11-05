//! Derived consumed-fact postings and source-position-specific output marks.

pub(crate) mod commit_touches;
pub(crate) mod computation_changes;
pub(in crate::domain_computation::primary_graph) use computation_changes::ComputationChanges;
pub(in crate::domain_computation::primary_graph) use computation_changes::ComputationRegistration;
pub(in crate::domain_computation::primary_graph) use computation_changes::PostconditionCarry;

pub(crate) mod delivery;
mod delivery_observation;
pub use delivery_observation::inexact_native_deliveries_on_this_thread_for_test;
pub(crate) mod derived;

mod equality;
mod equality_recovery;
pub(in crate::domain_computation::primary_graph) use equality_recovery::EqualityRecoveryRow;
pub(crate) mod fact_key;
mod fact_keys;

pub(crate) mod full_verification;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationDecision;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationImage;
pub(in crate::domain_computation::primary_graph) use full_verification::FullVerificationStop;
mod head_cell;
pub(in crate::domain_computation) use head_cell::HeadCellRegistrationStop;

mod equivalence_control;
pub(crate) mod logical_marking;
mod marking_observation;
pub use logical_marking::WorthQueryNativeMarkingPrecision;
pub use logical_marking::WorthQueryNativeMarkingWork;
pub use marking_observation::native_marking_on_this_thread_for_test;
pub use marking_observation::WorthQueryMarkedOutputForTest;
pub use marking_observation::WorthQueryNativeMarkingForTest;
pub(crate) mod mark_state;
mod output_availability;
pub(in crate::domain_computation::primary_graph) use output_availability::SourcePublicationReadFence;
pub(crate) mod output_facts;
pub(in crate::domain_computation) use output_availability::ObservedSourcePublication;
pub(in crate::domain_computation::primary_graph) use output_facts::RegisteredOutputFacts;

pub(crate) mod owner;
pub(crate) mod publication;
mod publication_cell;
pub(crate) mod query_coverage;
mod read_basis_carry;
pub(in crate::domain_computation::primary_graph) use query_coverage::QueryCoverageStop;
pub(in crate::domain_computation::primary_graph) use query_coverage::QueryFactCoverage;
mod reestablishment;

mod retirement;
pub(crate) mod settlement;
pub(crate) mod source_alignment;
mod source_publication;
mod touch_keys;
pub(crate) mod verification;
pub(crate) mod verified_current;

pub(in crate::domain_computation::primary_graph) use commit_touches::TouchedCommits;

pub(in crate::domain_computation::primary_graph) use derived::ConsumedOutputCurrentness;
pub(in crate::domain_computation::primary_graph) use derived::CurrentSettlementRegistrationCleanup;
pub(in crate::domain_computation::primary_graph) use derived::PreparedCurrentSettlementRegistration;

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
