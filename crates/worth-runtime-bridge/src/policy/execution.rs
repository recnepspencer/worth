//! Execution intent and the mandatory budget for serial Bridge delivery.
use super::{BridgeExecutionPolicyClass, BridgeRuntimePosture};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeExecutionPolicyBaseline {
    execution_class: BridgeExecutionPolicyClass,
    posture: BridgeRuntimePosture,
    request: worth_foundational::ExecutionRequestPolicy,
}

impl BridgeExecutionPolicyBaseline {
    pub const fn new(
        execution_class: BridgeExecutionPolicyClass,
        posture: BridgeRuntimePosture,
    ) -> Self {
        Self {
            execution_class,
            posture,
            request: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    64 << 20,
                    8_000_000,
                ),
            ),
        }
    }

    pub const fn operational() -> Self {
        Self::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Operational,
        )
    }

    pub const fn development() -> Self {
        Self::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Development,
        )
    }

    pub const fn forensic() -> Self {
        Self::new(
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeRuntimePosture::Forensic,
        )
    }

    pub fn execution_class(&self) -> BridgeExecutionPolicyClass {
        self.execution_class
    }

    pub fn posture(&self) -> BridgeRuntimePosture {
        self.posture
    }
    pub const fn request_policy(self) -> worth_foundational::ExecutionRequestPolicy {
        self.request
    }
}
