use crate::data::node::InstalledSignalConditionIdentity;
use crate::logic::evaluation::ConditionEvaluationContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstalledSignalConditionDecision {
    Eligible,
    Suppressed,
    Deferred,
}

pub trait InstalledSignalConditionResolver {
    fn resolve(
        &mut self,
        identity: &InstalledSignalConditionIdentity,
        context: &ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, crate::data::error::SignalError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalConditionalDecisionClass {
    ComputedChanged,
    ComputedRevertedClean,
    DependencyUnchanged,
    SuppressedBeforeCompute,
    DeferredByCondition,
    DeferredTemporal,
    DeferredOnDemand,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SignalConditionalDecisionCounters {
    pub request_admission_checks: usize,
    pub contract_lookups: usize,
    pub dependency_observation_reads: usize,
    pub dependency_version_checks: usize,
    pub condition_checks: usize,
    pub condition_deferrals: usize,
    pub temporal_deferrals: usize,
    pub on_demand_deferrals: usize,
    pub comparator_checks: usize,
    pub compute_contacts: usize,
    pub output_version_reads: usize,
    pub runtime_dependency_edges_captured: usize,
    pub application_contacts: usize,
    pub semantic_classifications: usize,
    pub reverted_clean_outcomes: usize,
    pub semantic_changes: usize,
    pub reuse_checks: usize,
    pub decisions_delivered: usize,
}

/// Signal-owned operational evidence. Fields are private so labels, digests,
/// or resolver return values cannot be restamped into execution authority.
pub struct SignalConditionalDecisionEvidence {
    pub(super) _authority: super::identity::SignalConditionalDecisionAuthorityIdentity,
    pub(super) projection: super::identity::SignalConditionalDecisionProjectionIdentity,
    pub(super) contract_authority:
        std::sync::Arc<super::contract::InstalledSignalConditionalAuthority>,
    pub(crate) attempt: u64,
    pub(crate) class: SignalConditionalDecisionClass,
    pub(crate) counters: SignalConditionalDecisionCounters,
    pub(crate) artifact_reuse_admitted: bool,
    pub(super) output_aspect: crate::data::aspect::Aspect,
    pub(super) output_version: u64,
    pub(super) _dependency_versions:
        super::dependency_versions::SignalConditionalDependencyVersions,
    pub(super) _execution: super::execution_proof::SignalConditionalExecutedRecipe,
}

impl SignalConditionalDecisionEvidence {
    pub fn projection(&self) -> &super::identity::SignalConditionalDecisionProjectionIdentity {
        &self.projection
    }
    pub const fn attempt(&self) -> u64 {
        self.attempt
    }
    pub const fn class(&self) -> SignalConditionalDecisionClass {
        self.class
    }
    pub const fn counters(&self) -> SignalConditionalDecisionCounters {
        self.counters
    }
    pub const fn artifact_reuse_admitted(&self) -> bool {
        self.artifact_reuse_admitted
    }
    pub const fn output_aspect(&self) -> crate::data::aspect::Aspect {
        self.output_aspect
    }
    pub const fn output_version(&self) -> u64 {
        self.output_version
    }
    pub fn dependency_version_count(&self) -> usize {
        self._dependency_versions.len()
    }
}
