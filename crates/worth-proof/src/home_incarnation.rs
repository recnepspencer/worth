//! Incarnation fencing never substitutes for the receiving owner's authority.
use std::num::NonZeroU64;

/// An immutable fence carried inside an owner-issued capability. This value
/// alone grants no access: the owner still checks its private capability axes.
/// It deliberately has no durable encoding or DurableIdentity implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HomeIncarnationFence {
    incarnation: NonZeroU64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_fences_refuse_both_directions_of_incarnation_drift() {
        // These are untrusted coordinates under comparison, not runtime or
        // capability fixtures. Matching coordinates alone grant no authority.
        let first = HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap());
        let next = HomeIncarnationFence::from_owner(NonZeroU64::new(2).unwrap());
        assert_eq!(
            next.admit(first),
            Err(HomeIncarnationDenial::StaleIncarnation)
        );
        assert_eq!(
            first.admit(next),
            Err(HomeIncarnationDenial::StaleIncarnation)
        );
        assert_eq!(next.admit(next), Ok(()));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeIncarnationDenial {
    ForeignHome,
    StaleIncarnation,
}

impl HomeIncarnationFence {
    /// Trusted carriage of the admitted incarnation inside the owner's sealed
    /// Home scope. The receiver admits that scope separately, before this fence.
    /// Constructing a fence cannot construct a capability or admit a runtime.
    pub const fn from_owner(incarnation: NonZeroU64) -> Self {
        Self { incarnation }
    }

    /// The receiver supplies its own current fence. Check before inspecting or
    /// acting on any capability payload from the caller's admission.
    pub fn admit(self, presented: Self) -> Result<(), HomeIncarnationDenial> {
        if self.incarnation != presented.incarnation {
            return Err(HomeIncarnationDenial::StaleIncarnation);
        }
        Ok(())
    }
}
