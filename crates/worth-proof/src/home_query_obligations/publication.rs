//! Terminal permission belongs to the original adapter and exact frame.
use super::*;
use crate::{AuthorityMarker, DurablyCommitted};

#[derive(Debug)]
pub struct HomeQueryObligationSubject {
    authority: HomeQueryObligationsAuthority,
    position: u64,
    payload_digest: [u8; 32],
}
impl HomeQueryObligationSubject {
    pub fn admit_to(
        &self,
        receiver: &HomeQueryObligationsAuthority,
    ) -> Result<(), HomeIncarnationDenial> {
        receiver.admit(&self.authority)
    }
    pub const fn position(&self) -> u64 {
        self.position
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
}

#[derive(Debug)]
pub struct CommittedHomeQueryObligation {
    subject: HomeQueryObligationSubject,
}
impl AuthorityMarker for CommittedHomeQueryObligation {}
impl CommittedHomeQueryObligation {
    pub const fn subject(&self) -> &HomeQueryObligationSubject {
        &self.subject
    }
}

/// No-effect custody cannot authorize a committed cutover.
/// ```compile_fail,E0308
/// use worth_proof::{CommittedHomeQueryObligation, DurablyCommitted, NoEffectHomeQueryObligation};
/// fn cutover(_: DurablyCommitted<CommittedHomeQueryObligation>) {}
/// fn wrong(no_effect: NoEffectHomeQueryObligation) { cutover(no_effect); }
/// ```
#[derive(Debug)]
pub struct NoEffectHomeQueryObligation {
    subject: HomeQueryObligationSubject,
}
impl NoEffectHomeQueryObligation {
    pub const fn subject(&self) -> &HomeQueryObligationSubject {
        &self.subject
    }
}

impl HomeQueryObligationsOwner {
    /// The Store binding calls this only while retaining actual committed custody.
    pub fn acknowledge_query_obligation_commit(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> DurablyCommitted<CommittedHomeQueryObligation> {
        DurablyCommitted::from_owner(CommittedHomeQueryObligation {
            subject: self.publication_subject(position, payload_digest),
        })
    }
    /// The original publication was proved unstarted or physically no-effect.
    pub fn acknowledge_query_obligation_no_effect(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> NoEffectHomeQueryObligation {
        NoEffectHomeQueryObligation {
            subject: self.publication_subject(position, payload_digest),
        }
    }
    fn publication_subject(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> HomeQueryObligationSubject {
        HomeQueryObligationSubject {
            authority: self.authority(),
            position,
            payload_digest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;
    #[test]
    fn commitment_keeps_the_original_subject_and_refuses_a_matching_foreign_issuer() {
        let fence = HomeIncarnationFence::from_owner(NonZeroU64::MIN);
        let owner = HomeQueryObligationsOwner::fresh([1; 16], fence);
        let foreign = HomeQueryObligationsOwner::fresh([1; 16], fence);
        owner
            .acknowledge_query_obligation_commit(7, [3; 32])
            .consume(|committed| {
                let subject = committed.subject();
                assert_eq!(subject.position(), 7);
                assert_eq!(subject.payload_digest(), [3; 32]);
                assert_eq!(subject.admit_to(&owner.authority()), Ok(()));
                assert_eq!(
                    subject.admit_to(&foreign.authority()),
                    Err(HomeIncarnationDenial::ForeignHome)
                );
            });
    }
    #[test]
    fn no_effect_custody_keeps_its_subject_and_refuses_the_next_incarnation() {
        let owner = HomeQueryObligationsOwner::fresh(
            [1; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::MIN),
        );
        let next = HomeQueryObligationsOwner::fresh(
            [1; 16],
            HomeIncarnationFence::from_owner(NonZeroU64::new(2).unwrap()),
        );
        let no_effect = owner.acknowledge_query_obligation_no_effect(9, [4; 32]);
        assert_eq!(no_effect.subject().position(), 9);
        assert_eq!(no_effect.subject().payload_digest(), [4; 32]);
        assert_eq!(no_effect.subject().admit_to(&owner.authority()), Ok(()));
        assert_eq!(
            no_effect.subject().admit_to(&next.authority()),
            Err(HomeIncarnationDenial::StaleIncarnation)
        );
    }
}
