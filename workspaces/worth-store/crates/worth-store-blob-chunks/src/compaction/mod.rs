#![doc = include_str!("compile_fail_proofs.md")]

//! # STATE GRAPH
//!
//! Compaction rewrite admission composes physical interlock, reachability, placement, and pacing
//! evidence before rewrite execution:
//!
//! - **Local read-plan completion** enters through released [`BlobCompactionReadHold`] matched to
//!   [`CompactionReadInterlockPlan`].
//! - **I/O execution authority** enters only when [`BlobCompactionIntentBasis`] consumes a
//!   scheduler-issued `BackgroundIdleCapacityLease` for compaction rewrite. An unpaced basis is
//!   not accepted by [`BlobCompactionAuthority::plan_compaction`].
//! - **Cold-tier posture** enters through [`BlobCompactionColdReadiness`] classified via tiering
//!   [`cold_posture_permits_compaction`].
//! - **Placement witness** must match lifecycle reachability via admitted [`AdmittedBlobPlacement`].

mod classification;
mod counters;
mod denial;
mod equivalence;
mod orchestration;
mod receipt_construction;
mod recovery;
mod rewrite_binding;
mod transitions;
mod types;
mod verification;

#[cfg(all(test, feature = "certification-test-authority"))]
mod read_plan_binding_tests;
#[cfg(all(test, feature = "certification-test-authority"))]
pub(crate) mod test_support;
#[cfg(all(test, feature = "certification-test-authority"))]
mod tests;

pub use counters::BlobCompactionCounterSnapshot;
pub use denial::BlobCompactionDenial;
pub use equivalence::BlobCompactionEquivalence;
pub use orchestration::BlobCompactionAuthority;
pub use receipt_construction::published_observation::BlobCompactionPublishedObservation;
pub use recovery::{BlobCompactionResidue, BlobCompactionRestartOutcome};
pub use transitions::execute_rewrite::BlobCompactionRewriteExecution;
pub use types::{
    BlobCompactionColdReadiness, BlobCompactionIntent, BlobCompactionIntentBasis,
    BlobCompactionPacingDenial, BlobCompactionPhysicalInterlock, BlobCompactionReadHold,
    BlobCompactionRewritePlan,
};
