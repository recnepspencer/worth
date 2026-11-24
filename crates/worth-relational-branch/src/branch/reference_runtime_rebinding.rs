use std::sync::{Arc, Mutex};
use worth_relational_contracts::internal_port::branch::identity::RelationalBranchIdentityOwnerPort as _;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

use worth_foundational::FoundationalBranchTarget;

use super::{
    relational_branch_observation, RelationalBranchObservationConstructionDenial,
    RelationalBranchReferenceCell, RelationalBranchReferenceMutableState,
};

impl RelationalBranchReferenceCell {
    pub fn rebind_runtime(
        &self,
        runtime_instance_id: u64,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        let state = self.state_snapshot();
        let target = match state.observation.target() {
            FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
            FoundationalBranchTarget::Basis(target) => FoundationalBranchTarget::basis(
                target.rebind_runtime_instance_id(runtime_instance_id),
            ),
        };
        let observation = relational_branch_observation(
            runtime_instance_id,
            self.identity.branch_id().0.as_str(),
            target,
            state.observation.generation(),
        )?;
        let fork_provenance = match (
            state.fork_provenance.as_ref(),
            state.fork_source_branch_id.as_ref(),
        ) {
            (None, None) => Ok(None),
            (Some(source), Some(source_branch_id)) => {
                let expected_branch_id = format!(
                    "relational/{}/{}",
                    self.identity.runtime_instance_id(),
                    source_branch_id.0
                );
                if source.branch_id().as_str() != expected_branch_id {
                    return Err(
                        RelationalBranchObservationConstructionDenial::ForkProvenanceMismatch,
                    );
                }
                let target = match source.target() {
                    FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
                    FoundationalBranchTarget::Basis(target) => FoundationalBranchTarget::basis(
                        target.rebind_runtime_instance_id(runtime_instance_id),
                    ),
                };
                relational_branch_observation(
                    runtime_instance_id,
                    &source_branch_id.0,
                    target,
                    source.generation(),
                )
                .map(Some)
            }
            _ => return Err(RelationalBranchObservationConstructionDenial::ForkProvenanceMismatch),
        }?;
        Ok(Self {
            identity: self.identity.rebind(runtime_instance_id),
            state: Arc::new(Mutex::new(RelationalBranchReferenceMutableState {
                observation,
                truth_version: state.truth_version,
                lifecycle: state.lifecycle,
                fork_provenance,
                fork_source_branch_id: state.fork_source_branch_id,
                root: state.root,
            })),
            basis_registry: crate::branch::basis_registry::RelationalBranchBasisRegistry::default(),
            coordination: crate::branch::coordination::RelationalBranchCoordinationCell::fresh(
                runtime_instance_id,
                self.identity.branch_id(),
            ),
            head_retention: crate::history::retention::RelationalBranchHeadRetentionCell::fresh(),
            sharing_costs: self.sharing_costs.detached_owner_snapshot(),
        })
    }
}
