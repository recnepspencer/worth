use worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract;
use worth_relational_contracts::internal_port::validation::data::execution::InvariantExecutionPoint;
pub use worth_relational_contracts::internal_port::validation::data::execution_posture::{InvariantExecutionDisposition, InvariantPlanScopeClass, InvariantScopeWideningCause};
use worth_relational_contracts::internal_port::validation::data::groups::{InvariantCostClass, InvariantGroupSet};
use worth_relational_contracts::internal_port::authority::commit::preparation::diagnostics::failures::PreparationFailureClass;
use worth_relational_contracts::internal_port::transactions::data::preparation_strategy::PreparationStrategy;
use serde::{Deserialize, Serialize};

use worth_relational_contracts::internal_port::validation::data::observation_kind::InvariantObservationKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantProofBoundarySummary {
    scope_class: InvariantPlanScopeClass,
    widened_causes: Vec<InvariantScopeWideningCause>,
    packet_count: usize,
    touched_partition_count: usize,
}

impl InvariantProofBoundarySummary {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.widened_causes.capacity() as u64)
            .saturating_mul(std::mem::size_of::<InvariantScopeWideningCause>() as u64)
    }

    pub fn new(
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantExecutionMetadata {
    execution_point: InvariantExecutionPoint,
    observation_kind: InvariantObservationKind,
    version_id: worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
    current_version_id:
        worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
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
    proposal_identity:
        Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
}

impl InvariantExecutionMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        execution_point: InvariantExecutionPoint,
        observation_kind: InvariantObservationKind,
        version_id: worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
        current_version_id: worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
        consumed_groups: InvariantGroupSet,
        applicable_groups: InvariantGroupSet,
        max_cost: InvariantCostClass,
        disposition: InvariantExecutionDisposition,
        plan_contract: Option<InvariantPlanContract>,
        has_merged_plan: bool,
        preparation_strategy: Option<PreparationStrategy>,
        preparation_failures: Vec<PreparationFailureClass>,
        proof_boundary: Option<InvariantProofBoundarySummary>,
        proposal_identity: Option<
            crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        >,
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

    pub fn execution_point(&self) -> InvariantExecutionPoint {
        self.execution_point
    }

    pub fn observation_kind(&self) -> InvariantObservationKind {
        self.observation_kind
    }

    pub fn version_id(
        &self,
    ) -> worth_relational_contracts::internal_port::identity::data::primitives::VersionId {
        self.version_id
    }

    pub fn current_version_id(
        &self,
    ) -> worth_relational_contracts::internal_port::identity::data::primitives::VersionId {
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

    pub fn proposal_identity(
        &self,
    ) -> Option<&crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>
    {
        self.proposal_identity.as_ref()
    }

    pub fn preparation_failures(&self) -> &[PreparationFailureClass] {
        &self.preparation_failures
    }
}
