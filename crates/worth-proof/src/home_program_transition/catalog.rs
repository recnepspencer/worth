//! Readmission of executable meaning is not permission to repeat an effect.
use super::*;

/// The admitting Home borrows its exact protected original intent data.
/// No step reservation or execution permission can be obtained from this seal.
#[derive(Debug)]
pub struct AdmittedHomeProgramTransitionCatalog<Source> {
    occurrence: Arc<TransitionOccurrence>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
    commitment: [u8; 32],
    source: Source,
}
impl<Source> AdmittedHomeProgramTransitionCatalog<Source> {
    pub fn source(&self) -> &Source {
        &self.source
    }
}
impl HomeProgramTransitionOwner {
    /// Called by the owner of the actual committed or readmitted intent.
    pub fn admit_catalog<Source>(
        &self,
        commitment: [u8; 32],
        source: Source,
    ) -> AdmittedHomeProgramTransitionCatalog<Source> {
        AdmittedHomeProgramTransitionCatalog {
            occurrence: self.occurrence.clone(),
            home: self.home,
            incarnation: self.incarnation,
            commitment,
            source,
        }
    }
}
impl HomeProgramTransitionAuthority {
    pub fn admit_catalog<Source>(
        &self,
        proof: &AdmittedHomeProgramTransitionCatalog<Source>,
        commitment: &[u8; 32],
    ) -> Result<(), HomeProgramTransitionAdmissionDenial> {
        self.admit(
            &proof.occurrence,
            proof.home,
            proof.incarnation,
            &proof.commitment,
            commitment,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    #[test]
    fn catalog_readmission_requires_its_attached_owner_incarnation_and_original_plan() {
        let owner = |incarnation| {
            HomeProgramTransitionOwner::fresh(
                [1; 16],
                HomeIncarnationFence::from_owner(NonZeroU64::new(incarnation).unwrap()),
            )
        };
        let original = owner(1);
        let proof = original.admit_catalog([7; 32], "protected original data");
        assert_eq!(proof.source(), &"protected original data");
        assert_eq!(original.authority().admit_catalog(&proof, &[7; 32]), Ok(()));
        assert_eq!(
            original.authority().admit_catalog(&proof, &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
        );
        assert_eq!(
            owner(1).authority().admit_catalog(&proof, &[7; 32]),
            Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
        );
        assert_eq!(
            owner(2).authority().admit_catalog(&proof, &[8; 32]),
            Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
        );
    }
}
