//! Query readmission stays bound to the original Home adapter occurrence.
use crate::{HomeIncarnationDenial, HomeIncarnationFence};
use std::sync::Arc;
mod publication;
pub use publication::{
    CommittedHomeQueryObligation, HomeQueryObligationSubject, NoEffectHomeQueryObligation,
};

#[derive(Debug)]
pub struct HomeQueryObligationsOwner {
    authority: HomeQueryObligationsAuthority,
}
#[derive(Clone, Debug)]
pub struct HomeQueryObligationsAuthority {
    occurrence: Arc<()>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}
#[derive(Clone, Debug)]
pub struct AdmittedHomeQueryObligations {
    authority: HomeQueryObligationsAuthority,
    position: u64,
    digest: [u8; 32],
    payload_digest: [u8; 32],
}
impl HomeQueryObligationsOwner {
    pub fn fresh(home: [u8; 16], incarnation: HomeIncarnationFence) -> Self {
        Self {
            authority: HomeQueryObligationsAuthority {
                occurrence: Arc::new(()),
                home,
                incarnation,
            },
        }
    }
    pub fn authority(&self) -> HomeQueryObligationsAuthority {
        self.authority.clone()
    }
    pub fn admit(&self, proof: &AdmittedHomeQueryObligations) -> Result<(), HomeIncarnationDenial> {
        self.authority.admit(&proof.authority)
    }
}
impl HomeQueryObligationsAuthority {
    pub fn admit(&self, presented: &Self) -> Result<(), HomeIncarnationDenial> {
        if self.home != presented.home {
            return Err(HomeIncarnationDenial::ForeignHome);
        }
        self.incarnation.admit(presented.incarnation)?;
        if !Arc::ptr_eq(&self.occurrence, &presented.occurrence) {
            return Err(HomeIncarnationDenial::ForeignHome);
        }
        Ok(())
    }
    /// Called by the Query owner only after decoding and reconciling the exact
    /// original protected prefix. A foreign issuer cannot admit to this Home.
    pub fn admit_query_obligation_prefix(
        &self,
        position: u64,
        digest: [u8; 32],
        payload_digest: [u8; 32],
    ) -> AdmittedHomeQueryObligations {
        AdmittedHomeQueryObligations {
            authority: self.clone(),
            position,
            digest,
            payload_digest,
        }
    }
}
impl AdmittedHomeQueryObligations {
    pub const fn position(&self) -> u64 {
        self.position
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;
    #[test]
    fn query_obligation_prefix_cannot_substitute_a_matching_foreign_issuer() {
        let fence = HomeIncarnationFence::from_owner(NonZeroU64::MIN);
        let actual = HomeQueryObligationsOwner::fresh([1; 16], fence);
        let foreign = HomeQueryObligationsOwner::fresh([1; 16], fence);
        let proof = foreign
            .authority()
            .admit_query_obligation_prefix(2, [3; 32], [4; 32]);
        assert_eq!(
            actual.admit(&proof),
            Err(HomeIncarnationDenial::ForeignHome)
        );
        let current = actual
            .authority()
            .admit_query_obligation_prefix(2, [3; 32], [4; 32]);
        assert_eq!(actual.admit(&current), Ok(()));
    }
    #[test]
    fn retained_query_obligation_admission_refuses_stale_incarnation() {
        let previous = HomeQueryObligationsOwner::fresh(
            [1; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::MIN),
        );
        let current = HomeQueryObligationsOwner::fresh(
            [1; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::new(2).unwrap()),
        );
        let proof = previous
            .authority()
            .admit_query_obligation_prefix(2, [3; 32], [4; 32]);
        assert_eq!(
            current.admit(&proof),
            Err(HomeIncarnationDenial::StaleIncarnation)
        );
        assert_eq!(proof.position(), 2);
    }
}
