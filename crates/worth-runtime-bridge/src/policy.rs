pub(crate) mod admission;
mod conditional_retention;
pub use conditional_retention::BridgeConditionalRetentionBudget;
mod contracts;
mod counters;
mod declaration;
mod lowering;
mod provenance;
mod rejection;
mod replay;
mod report;
mod taxonomy;
mod validation;

pub use contracts::{
    AdmittedBridgePolicyContract, AdmittedBridgePolicyContractParts, BridgePolicyAuthorityInputs,
    BridgePolicyResolutionEntry,
};
pub use counters::BridgePolicyCounters;
pub use declaration::{BridgePolicyDeclaration, BridgePolicyDeclarationIdentity};
pub use lowering::{BridgeRoutePlanningPolicy, LoweredBridgeExecutionPolicy};
pub use provenance::{BridgePolicyProvenanceEntry, BridgePolicyProvenanceRecord};
pub use rejection::{BridgePolicyRejection, BridgePolicyRejectionKind, BridgePolicyRejectionStage};
pub use replay::BridgePolicyReplayBundle;
pub use report::{BridgePolicyProvenanceReport, BridgePolicyProvenanceReportRow};
pub use taxonomy::{
    BridgeExecutionPolicyClass, BridgePolicyFieldKind, BridgePolicyResolution,
    BridgePolicySourceClass,
};
pub use validation::ValidatedBridgePolicyDeclaration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BridgeDiagnosticsTier {
    Minimal,
    #[default]
    Standard,
    Exhaustive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BridgeRuntimePosture {
    Operational,
    #[default]
    Development,
    Forensic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeDiagnosticsRetentionBudget {
    route_record_limit: usize,
    failure_record_limit: usize,
}

impl BridgeDiagnosticsRetentionBudget {
    pub const fn new(route_record_limit: usize, failure_record_limit: usize) -> Self {
        Self {
            route_record_limit,
            failure_record_limit,
        }
    }

    pub fn for_tier(tier: BridgeDiagnosticsTier) -> Self {
        match tier {
            BridgeDiagnosticsTier::Minimal => Self::new(32, 32),
            BridgeDiagnosticsTier::Standard => Self::new(128, 128),
            BridgeDiagnosticsTier::Exhaustive => Self::new(512, 512),
        }
    }

    pub fn route_record_limit(&self) -> usize {
        self.route_record_limit
    }

    pub fn failure_record_limit(&self) -> usize {
        self.failure_record_limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeExecutionPolicyBaseline {
    execution_class: BridgeExecutionPolicyClass,
    posture: BridgeRuntimePosture,
}

impl BridgeExecutionPolicyBaseline {
    pub const fn new(
        execution_class: BridgeExecutionPolicyClass,
        posture: BridgeRuntimePosture,
    ) -> Self {
        Self {
            execution_class,
            posture,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeDiagnosticsPolicyBaseline {
    diagnostics_tier: BridgeDiagnosticsTier,
    retention_budget: BridgeDiagnosticsRetentionBudget,
}

impl BridgeDiagnosticsPolicyBaseline {
    pub fn new(
        diagnostics_tier: BridgeDiagnosticsTier,
        retention_budget: BridgeDiagnosticsRetentionBudget,
    ) -> Self {
        Self {
            diagnostics_tier,
            retention_budget,
        }
    }

    pub fn for_tier(diagnostics_tier: BridgeDiagnosticsTier) -> Self {
        Self::new(
            diagnostics_tier,
            BridgeDiagnosticsRetentionBudget::for_tier(diagnostics_tier),
        )
    }

    pub fn diagnostics_tier(&self) -> BridgeDiagnosticsTier {
        self.diagnostics_tier
    }

    pub fn retention_budget(&self) -> BridgeDiagnosticsRetentionBudget {
        self.retention_budget
    }

    pub fn with_route_record_limit(mut self, route_record_limit: usize) -> Self {
        self.retention_budget = BridgeDiagnosticsRetentionBudget::new(
            route_record_limit.max(1),
            self.retention_budget.failure_record_limit(),
        );
        self
    }

    pub fn with_failure_record_limit(mut self, failure_record_limit: usize) -> Self {
        self.retention_budget = BridgeDiagnosticsRetentionBudget::new(
            self.retention_budget.route_record_limit(),
            failure_record_limit.max(1),
        );
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeArtifactPolicyBaseline {
    record_route_artifacts: bool,
    allow_replay_artifacts: bool,
}

impl BridgeArtifactPolicyBaseline {
    pub const fn new(record_route_artifacts: bool, allow_replay_artifacts: bool) -> Self {
        Self {
            record_route_artifacts,
            allow_replay_artifacts,
        }
    }

    pub const fn operational() -> Self {
        Self::new(true, true)
    }

    pub const fn development() -> Self {
        Self::new(true, true)
    }

    pub const fn forensic() -> Self {
        Self::new(true, true)
    }

    pub fn record_route_artifacts(&self) -> bool {
        self.record_route_artifacts
    }

    pub fn allow_replay_artifacts(&self) -> bool {
        self.allow_replay_artifacts
    }

    pub fn with_replay_artifacts(mut self, allow_replay_artifacts: bool) -> Self {
        self.allow_replay_artifacts = allow_replay_artifacts;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeRuntimePolicy {
    conditional_retention: BridgeConditionalRetentionBudget,
    execution: BridgeExecutionPolicyBaseline,
    diagnostics: BridgeDiagnosticsPolicyBaseline,
    artifacts: BridgeArtifactPolicyBaseline,
}

impl BridgeRuntimePolicy {
    pub const fn from_sections(
        execution: BridgeExecutionPolicyBaseline,
        diagnostics: BridgeDiagnosticsPolicyBaseline,
        artifacts: BridgeArtifactPolicyBaseline,
        conditional_retention: BridgeConditionalRetentionBudget,
    ) -> Self {
        Self {
            conditional_retention,
            execution,
            diagnostics,
            artifacts,
        }
    }

    pub const fn operational() -> Self {
        Self::from_sections(
            BridgeExecutionPolicyBaseline::operational(),
            BridgeDiagnosticsPolicyBaseline::for_tier_const(BridgeDiagnosticsTier::Minimal),
            BridgeArtifactPolicyBaseline::operational(),
            BridgeConditionalRetentionBudget::development(),
        )
    }

    pub const fn development() -> Self {
        Self::from_sections(
            BridgeExecutionPolicyBaseline::development(),
            BridgeDiagnosticsPolicyBaseline::for_tier_const(BridgeDiagnosticsTier::Standard),
            BridgeArtifactPolicyBaseline::development(),
            BridgeConditionalRetentionBudget::development(),
        )
    }

    pub const fn forensic() -> Self {
        Self::from_sections(
            BridgeExecutionPolicyBaseline::forensic(),
            BridgeDiagnosticsPolicyBaseline::for_tier_const(BridgeDiagnosticsTier::Exhaustive),
            BridgeArtifactPolicyBaseline::forensic(),
            BridgeConditionalRetentionBudget::development(),
        )
    }

    pub fn conditional_retention(&self) -> BridgeConditionalRetentionBudget {
        self.conditional_retention
    }

    pub fn with_conditional_retention(mut self, budget: BridgeConditionalRetentionBudget) -> Self {
        self.conditional_retention = budget;
        self
    }

    pub fn execution(&self) -> BridgeExecutionPolicyBaseline {
        self.execution
    }

    pub fn diagnostics(&self) -> BridgeDiagnosticsPolicyBaseline {
        self.diagnostics
    }

    pub fn artifacts(&self) -> BridgeArtifactPolicyBaseline {
        self.artifacts
    }

    pub fn diagnostics_tier(&self) -> BridgeDiagnosticsTier {
        self.diagnostics.diagnostics_tier()
    }

    pub fn retention_budget(&self) -> BridgeDiagnosticsRetentionBudget {
        self.diagnostics.retention_budget()
    }

    pub fn record_route_artifacts(&self) -> bool {
        self.artifacts.record_route_artifacts()
    }

    pub fn allow_replay_artifacts(&self) -> bool {
        self.artifacts.allow_replay_artifacts()
    }

    pub fn with_execution(mut self, execution: BridgeExecutionPolicyBaseline) -> Self {
        self.execution = execution;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: BridgeDiagnosticsPolicyBaseline) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_artifacts(mut self, artifacts: BridgeArtifactPolicyBaseline) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_route_record_limit(mut self, route_record_limit: usize) -> Self {
        self.diagnostics = self.diagnostics.with_route_record_limit(route_record_limit);
        self
    }

    pub fn with_failure_record_limit(mut self, failure_record_limit: usize) -> Self {
        self.diagnostics = self
            .diagnostics
            .with_failure_record_limit(failure_record_limit);
        self
    }

    pub fn with_replay_artifacts(mut self, allow_replay_artifacts: bool) -> Self {
        self.artifacts = self.artifacts.with_replay_artifacts(allow_replay_artifacts);
        self
    }
}

impl Default for BridgeRuntimePolicy {
    fn default() -> Self {
        Self::development()
    }
}

impl BridgeDiagnosticsPolicyBaseline {
    pub const fn for_tier_const(diagnostics_tier: BridgeDiagnosticsTier) -> Self {
        let retention_budget = match diagnostics_tier {
            BridgeDiagnosticsTier::Minimal => BridgeDiagnosticsRetentionBudget::new(32, 32),
            BridgeDiagnosticsTier::Standard => BridgeDiagnosticsRetentionBudget::new(128, 128),
            BridgeDiagnosticsTier::Exhaustive => BridgeDiagnosticsRetentionBudget::new(512, 512),
        };
        Self {
            diagnostics_tier,
            retention_budget,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BridgeArtifactPolicyBaseline, BridgeDiagnosticsPolicyBaseline, BridgeDiagnosticsTier,
        BridgeExecutionPolicyBaseline, BridgeExecutionPolicyClass, BridgeRuntimePolicy,
        BridgeRuntimePosture,
    };

    #[test]
    fn runtime_policy_from_sections_preserves_existing_accessors() {
        let policy = BridgeRuntimePolicy::from_sections(
            BridgeExecutionPolicyBaseline::new(
                BridgeExecutionPolicyClass::DeterministicCanonical,
                BridgeRuntimePosture::Development,
            ),
            BridgeDiagnosticsPolicyBaseline::for_tier(BridgeDiagnosticsTier::Standard)
                .with_route_record_limit(77)
                .with_failure_record_limit(19),
            BridgeArtifactPolicyBaseline::new(true, false),
            super::BridgeConditionalRetentionBudget::development(),
        );

        assert_eq!(
            policy.execution().execution_class(),
            BridgeExecutionPolicyClass::DeterministicCanonical
        );
        assert_eq!(policy.diagnostics_tier(), BridgeDiagnosticsTier::Standard);
        assert_eq!(policy.retention_budget().route_record_limit(), 77);
        assert_eq!(policy.retention_budget().failure_record_limit(), 19);
        assert!(policy.record_route_artifacts());
        assert!(!policy.allow_replay_artifacts());
    }
}
