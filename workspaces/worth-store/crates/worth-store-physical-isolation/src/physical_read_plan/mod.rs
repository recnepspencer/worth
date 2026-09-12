mod barrier;
#[cfg(any(test, feature = "certification-authority"))]
mod certification;
mod completion;
#[cfg(test)]
mod completion_tests;
mod counters;
mod denial;
mod footprint;
mod footprint_ranges;
mod handle;
mod hazard;
mod intent;
mod known_footprint_admission;
mod plan;
mod release;
mod retry;
mod root_observation;
mod scratch;
mod traversal;

pub use barrier::PhysicalReadReachabilityBarrier;
#[cfg(any(test, feature = "certification-authority"))]
pub(crate) use certification::read_plan_completion_for_certification_root;
#[cfg(any(test, feature = "certification-authority"))]
pub use certification::{
    stable_physical_read_plan_for_certification_seed,
    stable_physical_read_plan_for_certification_test,
};
pub use completion::PhysicalReadPlanCompletionReceipt;
pub use counters::ReadPlanCounterSnapshot;
pub use denial::PhysicalReadPlanAdmissionDenial;
pub use footprint::{
    CompactProtectedReferenceSet, PhysicalReadPlanFootprint, PhysicalReadProtectedFootprintBasis,
    ProtectedPhysicalReference, ProtectedPhysicalReferenceSet,
};
pub use footprint_ranges::{ProtectedReferenceRange, ProtectedReferenceRangeSet};
pub use handle::StablePhysicalReadHandle;
pub use hazard::PublishedReaderHazard;
pub use intent::UnprotectedReadIntent;
pub(crate) use known_footprint_admission::admit_known_footprint_read;
pub(crate) use plan::PhysicalReadPlanCompletion;
pub use plan::{
    admit_seed_stable_read_plan, physical_epoch_vector_for_current_root, SeedStableReadPlan,
    StablePhysicalReadPlan, StablePhysicalReadPlanAdmission,
};
pub use release::{PhysicalReadPlanReleaseReceipt, PhysicalReadPlanReleaseSemantics};
pub use retry::PhysicalReadPlanRetryPosture;
pub use root_observation::{
    PostProtectionPhysicalReadObservation, ProtectedRootObservation, ValidatedRootObservation,
};
pub use scratch::{ReadPlanAdmissionScratchArena, ReadPlanScratchUsage};
pub use traversal::{StepwiseStableReadCursor, TraversalAdmissionGuard, TraversalAdmissionReceipt};
