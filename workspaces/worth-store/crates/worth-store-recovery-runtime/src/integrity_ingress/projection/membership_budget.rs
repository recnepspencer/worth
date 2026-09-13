use crate::integrity_ingress::RecoveryIntegrityIngressRejection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MembershipProjectionFailure {
    Integrity(RecoveryIntegrityIngressRejection),
    EntryLimit { observed: u64 },
}

impl From<RecoveryIntegrityIngressRejection> for MembershipProjectionFailure {
    fn from(rejection: RecoveryIntegrityIngressRejection) -> Self {
        Self::Integrity(rejection)
    }
}

/// Bounds recovery-owned entry materialization after the fixed-page integrity check.
pub(super) fn admit_membership_entries(
    observed: usize,
    node_capacity: u16,
    remaining: u64,
) -> Result<(), MembershipProjectionFailure> {
    let observed = observed as u64;
    if observed > u64::from(node_capacity) {
        return Err(RecoveryIntegrityIngressRejection::NonCanonicalEncoding.into());
    }
    if observed > remaining {
        return Err(MembershipProjectionFailure::EntryLimit { observed });
    }
    Ok(())
}
