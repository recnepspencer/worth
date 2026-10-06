use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::{InvariantCostClass, InvariantGroupSet, InvariantPlanContract};
use crate::validation::engine::{
    InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantObservationKind,
    InvariantRequestProfile,
};
use crate::validation::invariant_access::InvariantAccess;

impl<'runtime> InvariantAccess<'runtime> {
    pub(super) fn execution_metadata(
        &self,
        profile: InvariantRequestProfile,
        observation_kind: InvariantObservationKind,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        merged_plan: Option<&MergedCommitPlan>,
        plan_contract: Option<InvariantPlanContract>,
        applicable_groups: InvariantGroupSet,
        max_cost: InvariantCostClass,
        disposition: InvariantExecutionDisposition,
        proposal_identity: Option<&crate::mvcc::RelationalMutationProposalIdentity>,
    ) -> InvariantExecutionMetadata {
        InvariantExecutionMetadata::new(
            profile.execution_point(),
            observation_kind,
            version_id,
            current_version_id,
            profile.consumed_groups(),
            applicable_groups,
            max_cost,
            disposition,
            plan_contract,
            merged_plan.is_some(),
            None,
            Vec::new(),
            None,
            proposal_identity.cloned(),
        )
    }
}
