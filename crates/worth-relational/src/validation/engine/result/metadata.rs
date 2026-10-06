use crate::validation::data::{
    InvariantCostClass, InvariantExecutionPoint, InvariantGroupSet, InvariantPlanContract,
};
use crate::{
    authority::commit::preparation::diagnostics::failures::PreparationFailureClass,
    authority::commit::preparation::planning::strategy::PreparationStrategy,
};
use serde::{Deserialize, Serialize};

use crate::validation::engine::InvariantObservationKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantPlanScopeClass {
    TouchedScope,
    PartitionScope,
    BroaderScope,
}

impl InvariantPlanScopeClass {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::TouchedScope => "touched_scope",
            Self::PartitionScope => "partition_scope",
            Self::BroaderScope => "broader_scope",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantScopeWideningCause {
    AllObservedPartitionScope,
    FullObservedReadSet,
}

impl InvariantScopeWideningCause {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::AllObservedPartitionScope => "all_observed_partition_scope",
            Self::FullObservedReadSet => "full_observed_read_set",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantProofBoundarySummary {
    scope_class: InvariantPlanScopeClass,
    widened_causes: Vec<InvariantScopeWideningCause>,
    packet_count: usize,
    touched_partition_count: usize,
}

impl InvariantProofBoundarySummary {
    pub(crate) fn new(
        scope_class: InvariantPlanScopeClass,
        widened_causes: Vec<InvariantScopeWideningCause>,
        packet_count: usize,
        touched_partition_count: usize,
    ) -> Self {
        Self {
            scope_class,
            widened_causes,
            packet_count,
            touched_partition_count,
        }
    }

    pub fn scope_class(&self) -> InvariantPlanScopeClass {
        self.scope_class
    }

    pub fn widened_causes(&self) -> &[InvariantScopeWideningCause] {
        &self.widened_causes
    }

    pub fn packet_count(&self) -> usize {
        self.packet_count
    }

    pub fn touched_partition_count(&self) -> usize {
        self.touched_partition_count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantExecutionDisposition {
    Executed,
    SkippedByPlanContract,
    SkippedByMayBreakMask,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantExecutionMetadata {
    execution_point: InvariantExecutionPoint,
    observation_kind: InvariantObservationKind,
    version_id: crate::identity::data::VersionId,
    current_version_id: crate::identity::data::VersionId,
    consumed_groups: InvariantGroupSet,
    applicable_groups: InvariantGroupSet,
    max_cost: InvariantCostClass,
    disposition: InvariantExecutionDisposition,
    plan_contract: Option<InvariantPlanContract>,
    has_merged_plan: bool,
    preparation_strategy: Option<PreparationStrategy>,
    preparation_failures: Vec<PreparationFailureClass>,
    proof_boundary: Option<InvariantProofBoundarySummary>,
    #[serde(skip)]
    proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}

impl InvariantExecutionMetadata {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        execution_point: InvariantExecutionPoint,
        observation_kind: InvariantObservationKind,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        consumed_groups: InvariantGroupSet,
        applicable_groups: InvariantGroupSet,
        max_cost: InvariantCostClass,
        disposition: InvariantExecutionDisposition,
        plan_contract: Option<InvariantPlanContract>,
        has_merged_plan: bool,
        preparation_strategy: Option<PreparationStrategy>,
        preparation_failures: Vec<PreparationFailureClass>,
        proof_boundary: Option<InvariantProofBoundarySummary>,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self {
            execution_point,
            observation_kind,
            version_id,
            current_version_id,
            consumed_groups,
            applicable_groups,
            max_cost,
            disposition,
            plan_contract,
            has_merged_plan,
            preparation_strategy,
            preparation_failures,
            proof_boundary,
            proposal_identity,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn executed_with_strategy(
        execution_point: InvariantExecutionPoint,
        observation_kind: InvariantObservationKind,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        consumed_groups: InvariantGroupSet,
        applicable_groups: InvariantGroupSet,
        max_cost: InvariantCostClass,
        plan_contract: Option<InvariantPlanContract>,
        has_merged_plan: bool,
        preparation_strategy: PreparationStrategy,
        preparation_failures: Vec<PreparationFailureClass>,
        proof_boundary: Option<InvariantProofBoundarySummary>,
        proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> Self {
        Self::new(
            execution_point,
            observation_kind,
            version_id,
            current_version_id,
            consumed_groups,
            applicable_groups,
            max_cost,
            InvariantExecutionDisposition::Executed,
            plan_contract,
            has_merged_plan,
            Some(preparation_strategy),
            preparation_failures,
            proof_boundary,
            proposal_identity,
        )
    }

    pub fn execution_point(&self) -> InvariantExecutionPoint {
        self.execution_point
    }

    pub fn observation_kind(&self) -> InvariantObservationKind {
        self.observation_kind
    }

    pub fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.current_version_id
    }

    pub fn consumed_groups(&self) -> InvariantGroupSet {
        self.consumed_groups
    }

    pub fn applicable_groups(&self) -> InvariantGroupSet {
        self.applicable_groups
    }

    pub fn max_cost(&self) -> InvariantCostClass {
        self.max_cost
    }

    pub fn disposition(&self) -> InvariantExecutionDisposition {
        self.disposition
    }

    pub fn plan_contract(&self) -> Option<InvariantPlanContract> {
        self.plan_contract
    }

    pub fn has_merged_plan(&self) -> bool {
        self.has_merged_plan
    }

    pub fn preparation_strategy(&self) -> Option<PreparationStrategy> {
        self.preparation_strategy
    }

    pub fn proof_boundary(&self) -> Option<&InvariantProofBoundarySummary> {
        self.proof_boundary.as_ref()
    }

    pub fn proposal_identity(&self) -> Option<&crate::mvcc::RelationalMutationProposalIdentity> {
        self.proposal_identity.as_ref()
    }

    pub(crate) fn preparation_failures(&self) -> &[PreparationFailureClass] {
        &self.preparation_failures
    }
}
