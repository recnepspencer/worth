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

    /// The work a checked build of this plan's tree charges when it
    /// completes, from its shape alone: `9n - L - R` for `n` partitions,
    /// where `L` and `R` are the lengths of the tree's left and right spines.
    /// Placing the leaves costs `n` visits, the monotone-stack pass `n` plus
    /// one per pop (`n - R`) and one per stop (`n - L`), the split plan `n`,
    /// and evaluation two visits and two combines per node. `None` when it
    /// does not fit.
    pub fn checked_build_work(&self) -> Option<u64> {
        let identities = self.identities();
        let spine = |priorities: &mut dyn Iterator<Item = (u64, PartitionIdentity)>| {
            let mut least = None;
            priorities.fold(0_u64, |count, priority| {
                if least.is_none_or(|least| priority < least) {
                    least = Some(priority);
                    count + 1
                } else {
                    count
                }
            })
        };
        let left = spine(&mut identities.iter().copied().map(priority));
        let right = spine(&mut identities.iter().rev().copied().map(priority));
        u64::try_from(identities.len())
            .ok()?
            .checked_mul(9)?
            .checked_sub(left)?
            .checked_sub(right)
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
