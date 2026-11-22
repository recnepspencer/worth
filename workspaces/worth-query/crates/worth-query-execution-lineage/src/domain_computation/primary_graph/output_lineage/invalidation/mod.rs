//! Derived consumed-fact postings and source-position-specific output marks.

pub mod commit_touches;
pub mod computation_changes;
pub use computation_changes::ComputationChanges;
pub use computation_changes::ComputationRegistration;
pub use computation_changes::PostconditionCarry;

pub mod delivery;
pub mod delivery_observation;
pub use delivery_observation::inexact_native_deliveries_on_this_thread_for_test;
pub mod derived;

pub mod equality;
pub mod equality_recovery;
pub use equality_recovery::EqualityRecoveryRow;
pub mod fact_key;
pub mod fact_keys;

pub mod full_verification;
pub use full_verification::FullVerificationDecision;
pub use full_verification::FullVerificationImage;
pub use full_verification::FullVerificationStop;
pub mod head_cell;
pub use head_cell::HeadCellRegistrationStop;

pub mod equivalence_control;
pub mod logical_marking;
pub mod marking_observation;
pub use logical_marking::WorthQueryNativeMarkingPrecision;
pub use logical_marking::WorthQueryNativeMarkingWork;
pub use marking_observation::native_marking_on_this_thread_for_test;
pub use marking_observation::WorthQueryMarkedOutputForTest;
pub use marking_observation::WorthQueryNativeMarkingForTest;
pub mod mark_state;
pub mod output_availability;
pub use output_availability::SourcePublicationReadFence;
pub mod output_facts;

pub use output_facts::RegisteredOutputFacts;

pub mod owner;
pub mod publication;
pub mod publication_cell;
pub mod query_coverage;
pub mod read_basis_carry;
pub use query_coverage::QueryCoverageStop;
pub use query_coverage::QueryFactCoverage;
pub mod reestablishment;

pub mod retirement;
pub mod settlement;
pub mod source_alignment;
pub mod source_publication;
pub mod touch_keys;
pub mod verification;
pub mod verified_current;

pub use commit_touches::CommitTouchEntities;
pub use commit_touches::CommitTouchInterest;
pub use commit_touches::TouchedCommits;

pub use derived::ConsumedOutputCurrentness;
pub use derived::CurrentSettlementRegistrationCleanup;
pub use derived::PreparedCurrentSettlementRegistration;
pub use derived::SettlementRegistrationStop;
pub use derived::SourceSettlementCurrentness;

pub use mark_state::FullVerificationReason;
pub use owner::SourceInvalidationOwner;
pub use publication::collect_consumed_output_upstream;
pub use publication::register_completed;
pub use publication::register_republished;
pub use settlement::PostingContinuation;
pub use settlement::PostingRebaseBasis;
pub use settlement::SettlementRegistration;
pub use verification::DirtyReverification;
pub use verification::SettlementVerificationStop;
pub use verified_current::VerifiedCurrentCleanup;

pub use output_availability::ObservedSourcePublication;
