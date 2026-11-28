use std::collections::BTreeSet;

use crate::state::SignalBranchId;

use super::super::runtime_state::SignalRuntime;
use super::{SignalBranchRetirementDenial, SignalBranchRetirementRequest};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn validate_retirement_request_after_with_admitted(
        &self,
        request: &SignalBranchRetirementRequest,
        retired_before: &BTreeSet<SignalBranchId>,
        admitted_allowance: u32,
    ) -> Result<(), SignalBranchRetirementDenial> {
        let branch_id = request.branch().id;
        if branch_id == self.graph.current_branch().id {
            return Err(SignalBranchRetirementDenial::CurrentBranch { branch_id });
        }
        let Some(live_branch) = self.branches.branch_handle(branch_id) else {
            return Err(SignalBranchRetirementDenial::UnknownBranch { branch_id });
        };
        if live_branch.parent_branch_id.is_none() {
            return Err(SignalBranchRetirementDenial::CanonicalBranch { branch_id });
        }
        let observed = self
            .observe_branch_transaction_head(&live_branch)
            .expect("validated stored branch must expose a transaction head");
        if request.expected_head() != &observed {
            return Err(SignalBranchRetirementDenial::StaleBranchHead {
                expected_generation: request.expected_head().generation(),
                observed_generation: observed.generation(),
            });
        }
        let children = self
            .branches
            .branch_children(branch_id)
            .into_iter()
            .filter(|child_id| !retired_before.contains(child_id))
            .collect::<Vec<_>>();
        if !children.is_empty() {
            return Err(SignalBranchRetirementDenial::LiveChildren {
                branch_id,
                child_branch_ids: children,
            });
        }
        let active_leases = self.branches.branch_retention_count(branch_id);
        if active_leases != 0 {
            return Err(SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases,
            });
        }
        let admitted_leases = self.branches.branch_admitted_retention_count(branch_id);
        if admitted_leases > admitted_allowance {
            return Err(SignalBranchRetirementDenial::RetainedAdmittedBasis {
                branch_id,
                active_leases: admitted_leases,
            });
        }
        Ok(())
    }
}
