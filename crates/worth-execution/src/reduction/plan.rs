use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionDenial {
    IdentitiesNotCanonical,
    ValueCountMismatch,
    CoverageMismatch,
    UnknownIdentity(PartitionIdentity),
    IdentityAlreadyPresent(PartitionIdentity),
    InvalidCanonicalEncoding,
    ReducerPanic,
    ResultCapacityExceeded,
    WorkCounterOverflow,
}

/// Checked identities for one canonical reduction. Fields are private so a
/// caller cannot claim a sorted, unique set without passing admission.
///
/// ```compile_fail
/// use worth_execution::ReductionPlan;
/// let _forged = ReductionPlan {};
/// ```
///
/// ```
/// use worth_execution::ReductionPlan;
/// use worth_foundational::PartitionIdentity;
/// let checked = ReductionPlan::try_from_sorted_unique(vec![PartitionIdentity::new(1)]).unwrap();
/// assert_eq!(checked.identities().len(), 1);
/// ```
pub struct ReductionPlan {
    identities: CanonicalUniqueVec<PartitionIdentity>,
}

impl ReductionPlan {
    pub fn try_from_sorted_unique(
        identities: Vec<PartitionIdentity>,
    ) -> Result<Self, ReductionDenial> {
        let identities = CanonicalUniqueVec::try_from_sorted_unique(identities)
            .map_err(|_| ReductionDenial::IdentitiesNotCanonical)?;
        Ok(Self { identities })
    }

    /// Keys already checked by construction need no second check.
    pub fn from_canonical(identities: CanonicalUniqueVec<PartitionIdentity>) -> Self {
        Self { identities }
    }

    pub fn identities(&self) -> &[PartitionIdentity] {
        self.identities.as_slice()
    }
}

/// SplitMix64 finalizer over the identity's portable numeric representation.
/// This is a fixed shape rule, not a process-randomized hash.
pub(super) fn priority(identity: PartitionIdentity) -> (u64, PartitionIdentity) {
    let mut value = identity.value().wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (value ^ (value >> 31), identity)
}
