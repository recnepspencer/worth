use std::collections::BTreeSet;

use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, PlannedSignalBranchRetirement,
    SignalBranchObservation, SignalBranchRetirementDenial, SignalBranchRetirementReason,
};
use crate::logic::transaction::canonical_digest;
use crate::state::{SignalBranchHandle, SignalBranchId};

use super::super::operation_control::SignalOwnerOperationBoundary;
use super::super::SignalOwnerOperationAdmission;
use super::retirement_reservation::map_retirement_registry_denial;
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn plan_retirement_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        self.plan_retirement_with_admitted_allowance_after(
            admission,
            expected,
            reason,
            1,
            &BTreeSet::new(),
        )
    }

    pub(in crate::branch::owner_services) fn plan_retirement_releasing_snapshots_exact(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        expected: AdmittedSignalBranchBasis,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let branch_id = expected.branch_id();
        let admitted_allowance =
            match self.retirement_snapshot_allowance(branch_id, releasing_snapshots) {
                Ok(allowance) => allowance,
                Err(denial) => return TransitionOutcome::denied(denial),
            };
        self.plan_retirement_with_admitted_allowance_after(
            admission,
            expected,
            reason,
            admitted_allowance,
            &BTreeSet::new(),
        )
    }

    fn plan_retirement_with_admitted_allowance_after(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
        admitted_allowance: u32,
        retired_before: &BTreeSet<SignalBranchId>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        if !self.basis_has_owner_affinity(&expected) {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let branch_id = expected.branch_id();
        let metadata = match self
            .metadata
            .retirement_planning_facts(admission, branch_id)
        {
            Ok(metadata) => metadata,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        let retention = self.retirement_retention_counts(branch_id);
        let cell = match self.lookup_cell(admission, branch_id) {
            Ok(cell) => cell,
            Err(denial) => {
                return TransitionOutcome::denied(map_retirement_registry_denial(denial, branch_id))
            }
        };
        let cell = match cell.retirement_planning_facts(admission) {
            Ok(cell) => cell,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::ExactBasisPreflight);
        self.finish_retirement_plan(
            expected,
            reason,
            admitted_allowance,
            retired_before,
            RetirementPlanningEvidence {
                branch: cell.branch,
                observation: cell.observation,
                child_branch_ids: metadata.child_branch_ids,
                merge_participant: metadata.merge_participant,
                external_retentions: retention.external,
                admitted_retentions: retention.admitted_or_reserved,
            },
        )
    }

    pub(in crate::branch::owner_services) fn plan_legacy_retirement_after(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch: SignalBranchHandle,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
        admitted_allowance: u32,
        retired_before: &BTreeSet<SignalBranchId>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let branch_id = branch.id;
        let cell = match self.lookup_cell(admission, branch_id) {
            Ok(cell) => cell,
            Err(denial) => {
                return TransitionOutcome::denied(map_retirement_registry_denial(denial, branch_id))
            }
        };
        let cell = match cell.retirement_planning_facts(admission) {
            Ok(cell) => cell,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        if cell.observation.compare(expected.observation()).is_err() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        if !self.basis_has_owner_affinity(&expected) {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let metadata = match self
            .metadata
            .retirement_planning_facts(admission, branch_id)
        {
            Ok(metadata) => metadata,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        let retention = self.retirement_retention_counts(branch_id);
        admission.reach_operation_boundary(SignalOwnerOperationBoundary::ExactBasisPreflight);
        self.finish_retirement_plan(
            expected,
            reason,
            admitted_allowance,
            retired_before,
            RetirementPlanningEvidence {
                branch: cell.branch,
                observation: cell.observation,
                child_branch_ids: metadata.child_branch_ids,
                merge_participant: metadata.merge_participant,
                external_retentions: retention.external,
                admitted_retentions: retention.admitted_or_reserved,
            },
        )
    }

    fn finish_retirement_plan(
        &self,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
        admitted_allowance: u32,
        retired_before: &BTreeSet<SignalBranchId>,
        evidence: RetirementPlanningEvidence,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        let branch_id = expected.branch_id();

        if evidence
            .observation
            .compare(expected.observation())
            .is_err()
        {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        if branch_id == self.selected_branch_id() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CurrentBranch {
                branch_id,
            });
        }
        if evidence.branch.parent_branch_id.is_none() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBranch {
                branch_id,
            });
        }
        let blocking_children = evidence
            .child_branch_ids
            .iter()
            .copied()
            .filter(|child_id| !retired_before.contains(child_id))
            .collect::<Vec<_>>();
        if !blocking_children.is_empty() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::LiveChildren {
                branch_id,
                child_branch_ids: blocking_children,
            });
        }
        if evidence.merge_participant {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::MergeParticipant {
                branch_id,
            });
        }
        if evidence.external_retentions != 0 {
            return TransitionOutcome::denied(
                SignalBranchRetirementDenial::RetainedComponentBasis {
                    branch_id,
                    active_leases: evidence.external_retentions,
                },
            );
        }
        if evidence.admitted_retentions != admitted_allowance {
            return TransitionOutcome::denied(
                SignalBranchRetirementDenial::RetainedAdmittedBasis {
                    branch_id,
                    active_leases: evidence.admitted_retentions,
                },
            );
        }
        let shared_holders = expected.shared_holder_count();
        if shared_holders != 1 {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
                branch_id,
                shared_holders,
            });
        }

        let terminal_basis_digest = canonical_digest(&expected.observation().canonical_encoding());
        TransitionOutcome::success(PlannedSignalBranchRetirement {
            branch: evidence.branch,
            reason,
            terminal_basis_digest,
            planned_child_membership_count: evidence.child_branch_ids.len() as u32,
            admitted_basis: expected,
        })
    }

    pub(in crate::branch::owner_services) fn retirement_snapshot_allowance(
        &self,
        branch_id: SignalBranchId,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
    ) -> Result<u32, SignalBranchRetirementDenial> {
        let expected_runtime_instance_id = self.runtime_instance_id();
        let mut unique_retentions = BTreeSet::new();
        for admitted_snapshot in releasing_snapshots {
            let observed_runtime_instance_id = admitted_snapshot.owner_runtime_instance_id();
            if observed_runtime_instance_id != expected_runtime_instance_id {
                return Err(SignalBranchRetirementDenial::ForeignRetirementSnapshot {
                    expected_runtime_instance_id,
                    observed_runtime_instance_id,
                });
            }
            let snapshot_branch_id = admitted_snapshot.snapshot().meta.branch_id;
            if snapshot_branch_id != branch_id {
                return Err(
                    SignalBranchRetirementDenial::RetirementSnapshotBranchMismatch {
                        branch_id,
                        snapshot_branch_id,
                    },
                );
            }
            unique_retentions.insert(admitted_snapshot.retention_identity());
        }
        Ok(1_u32.saturating_add(unique_retentions.len() as u32))
    }
}

struct RetirementPlanningEvidence {
    branch: SignalBranchHandle,
    observation: SignalBranchObservation,
    child_branch_ids: Vec<SignalBranchId>,
    merge_participant: bool,
    external_retentions: u32,
    admitted_retentions: u32,
}
