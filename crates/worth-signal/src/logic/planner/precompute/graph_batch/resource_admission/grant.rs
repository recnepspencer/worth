//! Capacity authority minted only after a checked graph epoch is admitted.
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::logic::planner::apply::workspace::ApplyMemberBasis;
use std::num::NonZeroUsize;

pub(in crate::logic::planner) struct CheckedEpochAdmissionGrant {
    width: NonZeroUsize,
    result_grant_bytes: Option<u64>,
    apply_bases: Vec<ApplyMemberBasis>,
}

pub(in crate::logic::planner) struct CheckedApplyCapacity {
    members: Vec<ApplyMemberCapacity>,
}

pub(in crate::logic::planner) struct ApplyMemberCapacity {
    scratch_bytes: u64,
    result_bytes: u64,
    policy: Option<VersionComparatorPolicy>,
}

impl CheckedEpochAdmissionGrant {
    pub(super) fn new(
        width: usize,
        result_grant_bytes: Option<u64>,
        apply_bases: Vec<ApplyMemberBasis>,
    ) -> Self {
        Self {
            width: NonZeroUsize::new(width).expect("admitted epoch contains a task"),
            result_grant_bytes,
            apply_bases,
        }
    }

    pub(in crate::logic::planner) fn width(&self) -> NonZeroUsize {
        self.width
    }

    pub(in crate::logic::planner) fn result_grant_bytes(&self) -> Option<u64> {
        self.result_grant_bytes
    }

    pub(in crate::logic::planner) fn into_parts(
        self,
        measured_result_bytes: u64,
    ) -> Result<(NonZeroUsize, CheckedApplyCapacity), SignalError> {
        Ok((
            self.width,
            CheckedApplyCapacity::from_bases(self.apply_bases, measured_result_bytes)?,
        ))
    }
}

impl CheckedApplyCapacity {
    pub(super) fn from_bases(
        bases: Vec<ApplyMemberBasis>,
        result_grant_bytes: u64,
    ) -> Result<Self, SignalError> {
        let members = bases
            .into_iter()
            .map(|basis| {
                let (scratch_bytes, result_bytes, policy) =
                    basis.into_capacity(result_grant_bytes)?;
                Ok(ApplyMemberCapacity {
                    scratch_bytes,
                    result_bytes,
                    policy: Some(policy),
                })
            })
            .collect::<Result<Vec<_>, SignalError>>()?;
        Ok(Self { members })
    }

    pub(in crate::logic::planner) fn members(&self) -> &[ApplyMemberCapacity] {
        &self.members
    }

    pub(in crate::logic::planner) fn take_policy(
        &mut self,
        task_index: usize,
    ) -> Result<VersionComparatorPolicy, SignalError> {
        self.members
            .get_mut(task_index)
            .and_then(|member| member.policy.take())
            .ok_or_else(|| SignalError::internal("admitted comparator policy was already consumed"))
    }
}

impl ApplyMemberCapacity {
    pub(in crate::logic::planner) fn scratch_bytes(&self) -> u64 {
        self.scratch_bytes
    }

    pub(in crate::logic::planner) fn result_bytes(&self) -> u64 {
        self.result_bytes
    }
}
