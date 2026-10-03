//! Cumulative preparation for the installed policy's owned lookup key.

use std::mem::size_of;

use worth_query_installation::facade::WorthQueryInstalledAbilityRequirement;

use super::{WorthQueryInstalledAuthorizationPolicy, WorthQueryInstalledAuthorizationRegistry};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

#[derive(Debug)]
pub(in crate::domain_computation::authorization) enum InstalledPolicyAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
    Policy(WorthQueryOperationAuthorizationDenial),
}

impl WorthQueryInstalledAuthorizationRegistry {
    /// Admit the actual three-string search key and installed path checks
    /// before the existing policy owner constructs or reads that key.
    pub(in crate::domain_computation::authorization) fn policy_admitted<Stop>(
        &self,
        requirement: &WorthQueryInstalledAbilityRequirement,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<&WorthQueryInstalledAuthorizationPolicy, InstalledPolicyAdmissionStop<Stop>> {
        use InstalledPolicyAdmissionStop as StopKind;
        let key_bytes = requirement
            .ability()
            .len()
            .checked_add(requirement.scope_entity().len())
            .and_then(|n| n.checked_add(requirement.policy().len()))
            .ok_or(StopKind::AccountingOverflow)?;
        // std BTree nodes contain at most eleven keys. log2 is a conservative
        // height ceiling; each comparison can inspect all three search texts.
        let levels = self
            .policies
            .len()
            .checked_add(1)
            .ok_or(StopKind::AccountingOverflow)?;
        let levels = usize::BITS as usize - levels.leading_zeros() as usize;
        let comparisons = levels
            .checked_mul(11)
            .and_then(|n| n.checked_mul(key_bytes.checked_add(3)?))
            .and_then(|n| n.checked_add(requirement.policy_paths().len()))
            .ok_or(StopKind::AccountingOverflow)?;
        let work = comparisons
            .checked_add(key_bytes)
            .and_then(|n| n.checked_add(requirement.policy().len()))
            .and_then(|n| n.checked_add(4))
            .ok_or(StopKind::AccountingOverflow)?;
        // The existing policy error also copies the policy subject and owns
        // one cause slot. Preclaim it here because the lookup may refuse.
        let backing = key_bytes
            .checked_add(requirement.policy().len())
            .and_then(|n| n.checked_add(size_of::<WorthQueryOperationAuthorizationDenialKind>()))
            .ok_or(StopKind::AccountingOverflow)?;
        admit(
            u64::try_from(work).map_err(|_| StopKind::AccountingOverflow)?,
            u64::try_from(backing).map_err(|_| StopKind::AccountingOverflow)?,
        )
        .map_err(StopKind::Admission)?;
        self.policy(requirement).map_err(StopKind::Policy)
    }
}
