use worth_foundational::facade::CanonicalDigestWorkBudget;
use worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements;

use crate::application_aftermath::{
    InstalledExternalEffectContract, WorthQueryInstalledAftermathContract,
};
use crate::application_operation::{
    WorthQueryInstalledAbilityRequirement, WorthQueryInstalledApplicationOperationAuthorization,
    WorthQueryInstalledApplicationOperationExecutionPosture,
    WorthQueryInstalledMutationPrecondition, WorthQueryOperationEmissionContract,
};
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::domain_computation::{
    WorthQueryExecutionResourceContract, WorthQueryExecutionStrategyContract,
};
use crate::domain_operation::{
    WorthQueryInvariantExecutionContract, WorthQueryOperationDecisionFactContract,
    WorthQueryOperationEffectContract, WorthQueryOperationGraphReadContract,
    WorthQueryOperationInvariantContract, WorthQueryOperationReadTouchOverlapIndex,
    WorthQueryOperationTouchContract,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCompiledApplicationOperationContracts {
    pub(super) authorization: WorthQueryInstalledApplicationOperationAuthorization,
    pub(super) ability_requirements: Vec<WorthQueryInstalledAbilityRequirement>,
    pub(super) graph_reads: WorthQueryOperationGraphReadContract,
    pub(super) touches: WorthQueryOperationTouchContract,
    pub(super) emissions: WorthQueryOperationEmissionContract,
    pub(super) effects: WorthQueryOperationEffectContract,
    pub(super) invariants: WorthQueryOperationInvariantContract,
    pub(super) decision_facts: WorthQueryOperationDecisionFactContract,
    pub(super) invariant_execution: WorthQueryInvariantExecutionContract,
    pub(super) resources: WorthQueryExecutionResourceContract,
    pub(super) decision_fact_budget: usize,
    pub(super) projection_work_budget: usize,
    pub(super) additional_authorization_fact_count: usize,
    pub(super) mutation_preconditions: Vec<WorthQueryInstalledMutationPrecondition>,
    pub(super) execution_posture: WorthQueryInstalledApplicationOperationExecutionPosture,
    pub(super) external_effect: InstalledExternalEffectContract,
    pub(super) aftermath: Option<WorthQueryInstalledAftermathContract>,
    pub(super) overlap_index: WorthQueryOperationReadTouchOverlapIndex,
    pub(super) platform_candidate_ceiling: Option<ApplicationCandidateRequirements>,
    pub(super) workflow_settlement_ceiling: Option<ApplicationCandidateRequirements>,
}

impl WorthQueryCompiledApplicationOperationContracts {
    /// The componentwise maximum declared by installed mutation bindings for
    /// this operation. Query-owned platform effects cannot exceed these kinds.
    pub const fn platform_candidate_ceiling(&self) -> Option<ApplicationCandidateRequirements> {
        self.platform_candidate_ceiling
    }

    /// Query-owned sidecar capacity for one locally performed workflow step.
    /// It is separate from the product handler's candidate writer ceiling.
    pub const fn workflow_settlement_ceiling(&self) -> Option<ApplicationCandidateRequirements> {
        self.workflow_settlement_ceiling
    }

    pub fn mutation_preconditions(&self) -> &[WorthQueryInstalledMutationPrecondition] {
        &self.mutation_preconditions
    }

    pub const fn external_effect(&self) -> &InstalledExternalEffectContract {
        &self.external_effect
    }

    pub const fn aftermath(&self) -> Option<&WorthQueryInstalledAftermathContract> {
        self.aftermath.as_ref()
    }

    pub const fn authorization(&self) -> WorthQueryInstalledApplicationOperationAuthorization {
        self.authorization
    }

    pub fn precondition_canonical_work_budget(&self) -> Option<CanonicalDigestWorkBudget> {
        let count = u32::try_from(self.mutation_preconditions.len()).ok()?;
        let entries = count.checked_mul(5)?.checked_add(1)?;
        CanonicalDigestWorkBudget::new(entries, 256 * 1_024)
    }

    pub fn delegation_activation_proposal_canonical_work_budget(
        &self,
    ) -> Option<CanonicalDigestWorkBudget> {
        if !self.execution_posture.requires_delegation_activation() {
            return None;
        }
        let width = u32::try_from(
            self.touches.scopes().len() + usize::from(self.external_effect.is_declared()),
        )
        .ok()?;
        let entries = width.checked_mul(6)?.checked_add(16)?;
        CanonicalDigestWorkBudget::new(entries, 256 * 1_024)
    }

    pub fn capability_revocation_proposal_canonical_work_budget(
        &self,
    ) -> Option<CanonicalDigestWorkBudget> {
        if !self.execution_posture.requires_capability_revocation() {
            return None;
        }
        CanonicalDigestWorkBudget::new(16, 64 * 1_024)
    }

    pub fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.ability_requirements.iter().fold(
            WorthQueryCanonicalWorkEvidence::zero(),
            |work, requirement| work.combine(requirement.canonical_work()),
        )
    }

    pub fn ability_requirements(&self) -> &[WorthQueryInstalledAbilityRequirement] {
        &self.ability_requirements
    }

    pub fn graph_reads(&self) -> &WorthQueryOperationGraphReadContract {
        &self.graph_reads
    }

    pub fn touches(&self) -> &WorthQueryOperationTouchContract {
        &self.touches
    }

    pub const fn emissions(&self) -> &WorthQueryOperationEmissionContract {
        &self.emissions
    }

    pub const fn read_touch_overlap(&self) -> &WorthQueryOperationReadTouchOverlapIndex {
        &self.overlap_index
    }

    pub fn effects(&self) -> &WorthQueryOperationEffectContract {
        &self.effects
    }

    pub fn invariants(&self) -> &WorthQueryOperationInvariantContract {
        &self.invariants
    }

    pub fn decision_facts(&self) -> &WorthQueryOperationDecisionFactContract {
        &self.decision_facts
    }

    pub fn invariant_execution(&self) -> &WorthQueryInvariantExecutionContract {
        &self.invariant_execution
    }

    pub fn resources(&self) -> &WorthQueryExecutionResourceContract {
        &self.resources
    }

    pub fn execution_strategy(&self) -> Option<&WorthQueryExecutionStrategyContract> {
        let [strategy] = self.resources.strategies() else {
            return None;
        };
        Some(strategy)
    }

    pub const fn decision_fact_budget(&self) -> usize {
        self.decision_fact_budget
    }

    pub const fn projection_work_budget(&self) -> usize {
        self.projection_work_budget
    }

    pub const fn execution_posture(
        &self,
    ) -> WorthQueryInstalledApplicationOperationExecutionPosture {
        self.execution_posture
    }
}
