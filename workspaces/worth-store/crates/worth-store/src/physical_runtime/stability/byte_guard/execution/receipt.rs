use super::{
    PhysicalReadIoPosture, StablePhysicalReadExecutionCounters,
    StablePhysicalReadFoundationalEvidence,
};
use worth_store_physical_isolation::{
    PhysicalReadPlanCompletionReceipt, PhysicalReadPlanReleaseReceipt,
};

/// Byte execution evidence cannot be constructed from local plan completion.
///
/// ```compile_fail,E0277
/// use worth_store_physical_isolation::PhysicalReadPlanCompletionReceipt;
/// use worth_store::physical_runtime::stability::StablePhysicalReadReceipt;
/// fn promote(plan: PhysicalReadPlanCompletionReceipt) -> StablePhysicalReadReceipt {
///     plan.into()
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StablePhysicalReadReceipt {
    plan_completion: PhysicalReadPlanCompletionReceipt,
    counters: StablePhysicalReadExecutionCounters,
    io_posture: PhysicalReadIoPosture,
}

impl StablePhysicalReadReceipt {
    pub(crate) const fn new(
        plan_completion: PhysicalReadPlanCompletionReceipt,
        counters: StablePhysicalReadExecutionCounters,
        io_posture: PhysicalReadIoPosture,
    ) -> Self {
        Self {
            plan_completion,
            counters,
            io_posture,
        }
    }

    pub const fn read_plan_release(self) -> PhysicalReadPlanReleaseReceipt {
        self.plan_completion.read_plan_release()
    }

    pub const fn plan_completion(self) -> PhysicalReadPlanCompletionReceipt {
        self.plan_completion
    }

    pub const fn counters(self) -> StablePhysicalReadExecutionCounters {
        self.counters
    }

    pub const fn io_posture(self) -> PhysicalReadIoPosture {
        self.io_posture
    }

    pub fn lower_to_foundational_evidence(
        &self,
    ) -> Result<
        StablePhysicalReadFoundationalEvidence,
        worth_foundational::FoundationalBoundaryEvidenceProvenanceConstructionDenial,
    > {
        StablePhysicalReadFoundationalEvidence::lower(self)
    }
}

#[cfg(feature = "certification-test-authority")]
pub fn stable_physical_read_receipt_for_certification_test(
    guarded_bytes: u64,
) -> StablePhysicalReadReceipt {
    let completed =
        worth_store_physical_isolation::stable_physical_read_plan_for_certification_test(
            guarded_bytes,
        )
        .into_execution_ready_handle()
        .complete_plan();
    StablePhysicalReadReceipt::new(
        completed,
        StablePhysicalReadExecutionCounters::for_certification_test(guarded_bytes),
        PhysicalReadIoPosture::ordinary(),
    )
}
