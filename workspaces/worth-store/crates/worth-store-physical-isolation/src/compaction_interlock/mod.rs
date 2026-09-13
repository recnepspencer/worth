//! Compaction interlock verdicts must consume executed lower evidence.
//!
//! A compaction read verdict cannot be minted from a plan alone:
//!
//! ```compile_fail
//! let _shortcut =
//!     worth_store_physical_isolation::execute_admitted_compaction_rewrite_for_plan;
//! ```

mod candidate_ranges;
mod counters;
mod cutover_delta;
mod denial;
mod foundational_evidence;
mod mutation_lane_receipt;
mod owner_case;
mod owner_inventory;
mod plan;
mod plan_identity;
mod protected_set;
mod publication;
mod reclaim_queue;
mod recovery_evidence;
mod scheduler_demand;
mod source_admission;
mod stability_proof;
mod verdict;

pub use candidate_ranges::CompactionCandidateRangeSet;
pub use counters::CompactionReadInterlockCounters;
pub use cutover_delta::CompactionCutoverDelta;
pub use denial::CompactionReadInterlockDenial;
pub use foundational_evidence::CompactionInterlockFoundationalEvidence;
pub use mutation_lane_receipt::{
    CompactionMutationLaneOrigin, CompactionMutationLaneReceipt, CompactionMutationLaneReceiptKind,
};
pub use owner_case::{
    CompactionCutoverState, CompactionOwnerCaseDeclaration, CompactionOwnerCaseId,
    CompactionOwnerCaseObservation,
};
pub use owner_inventory::compaction_owner_case_inventory;
pub use plan::{CompactionReadInterlockPlan, CompactionSourceIntegrityEvidence};
pub use protected_set::CompactionProtectedReferenceSet;
pub use publication::CompactionRewritePublication;
pub use reclaim_queue::{CompactionDeferredReclaimQueue, DrainedCompactionReclaim};
pub use recovery_evidence::CompactionRecoveryEvidence;
pub use scheduler_demand::compaction_rewrite_scheduler_demand;
pub use source_admission::{
    CompactionSourceIntegrityAdmission, CompactionSourceIntegrityAdmissionDenial,
};
pub use stability_proof::CompactionCutoverStabilityProof;
pub use verdict::{execute_read_during_compaction_cutover, ReadDuringCompactionVerdict};
