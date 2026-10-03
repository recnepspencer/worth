//! Derived consumed-fact postings and source-position-specific output marks.

mod admission;
mod consumed_capacity;
mod delivery;
mod derived;
mod edit_admission;
mod equality;
mod fact_key;
mod fact_keys;
mod fact_retention;
#[cfg(feature = "certification-invalidation-equivalence")]
mod full_verification;
#[cfg(feature = "certification-invalidation-equivalence")]
pub(in crate::domain_computation::primary_graph) use full_verification::{
    FullVerificationDecision, FullVerificationImage, FullVerificationStop,
};
mod index_capacity;
pub(in crate::domain_computation::primary_graph) use index_capacity::arc_bytes;
mod logical_marking;
mod mark_state;
mod output_facts;
mod output_witness_capacity;
mod owner;
mod publication;
mod retention;
mod settlement;
mod source_alignment;
mod touch_keys;
mod verification;
mod verified_current;

#[cfg(test)]
mod native_journey_tests;

pub(in crate::domain_computation::primary_graph) use consumed_capacity::RetainedConsumedOutputCapacity;
pub(in crate::domain_computation::primary_graph) use derived::{
    ConsumedOutputCurrentness, CurrentSettlementRegistrationCleanup,
    PreparedCurrentSettlementRegistration, SettlementRegistrationStop, SourceSettlementCurrentness,
};
pub(in crate::domain_computation) use edit_admission::InvalidationEditAdmission;
pub(in crate::domain_computation::primary_graph) use edit_admission::{
    CarriedRequestInvalidationAdmission, ReservedExternalWork,
};
pub(in crate::domain_computation::primary_graph::output_lineage) use fact_retention::arc_slice_bytes;

pub(in crate::domain_computation::primary_graph::output_lineage) fn retained_fact_payload_bytes(
    fact: &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    admission: &mut InvalidationEditAdmission,
) -> Result<u64, worth_relational::facade::mvcc::CompanionPreflightStop> {
    fact_retention::fact_payload_bytes(fact, admission)
}
pub(in crate::domain_computation::primary_graph) use mark_state::FullVerificationReason;
pub(in crate::domain_computation) use owner::SourceInvalidationOwner;
pub(in crate::domain_computation::primary_graph) use publication::collect_consumed_output_upstream;
pub(in crate::domain_computation::primary_graph) use publication::register_completed;
pub(in crate::domain_computation::primary_graph) use settlement::SettlementRegistration;
pub(in crate::domain_computation::primary_graph) use verification::{
    DirtyReverification, SettlementVerificationStop,
};
pub(in crate::domain_computation::primary_graph) use verified_current::VerifiedCurrentCleanup;
