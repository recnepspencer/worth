use std::sync::Arc;
mod historical_selection;
pub use historical_selection::AdmittedHistoricalOutputSelection;
mod source_scope_retirement;
pub use source_scope_retirement::AdmittedProducerOutputRetirement;

/// The installed application owner retained by the runtime that selects producers.
/// A fresh occurrence cannot satisfy a native binding issued for another owner.
#[derive(Debug)]
pub struct InstalledApplicationPublicationOwner {
    occurrence: Arc<()>,
}

#[derive(Clone, Debug)]
pub struct InstalledApplicationPublicationAuthority {
    occurrence: Arc<()>,
}

/// Move-only admission from the running, installed required-output producer.
/// This permits attribution of an attempt; it does not prove publication occurred.
#[derive(Debug)]
pub struct AdmittedRequiredOutputPublication {
    occurrence: Arc<()>,
    producer: Arc<str>,
    dependency_identity: [u8; 32],
}

/// Native publication identity. Descriptions cannot replace the installed issuer
/// or the runtime's performed canonical custody that accompanies this seal.
#[derive(Clone, Debug)]
pub struct RequiredOutputPublicationAuthority {
    occurrence: Arc<()>,
    publication: u64,
    ordinal: usize,
}

#[derive(Debug)]
pub struct AdmittedRequiredOutputSuccession {
    running: AdmittedRequiredOutputPublication,
    predecessor: RequiredOutputPublicationAuthority,
}

/// The installed ordinary author retires an exact performed publication.
/// This is not a running producer and carries no producer dependency claim.
#[derive(Debug)]
pub struct AdmittedAuthoredOutputRetirement {
    predecessors: Vec<RequiredOutputPublicationAuthority>,
}

/// Admission issued only after the installed owner authenticates its recovered
/// program activation predecessor. Native adoption still binds the exact patch.
#[derive(Debug)]
pub struct AdmittedCheckpointProgramActivation {
    occurrence: Arc<()>,
    commitment: [u8; 32],
}

impl RequiredOutputPublicationAuthority {
    /// Descriptive canonical coordinate; admission still requires issuer custody.
    pub fn publication_coordinate(&self) -> (u64, usize) {
        (self.publication, self.ordinal)
    }
}

impl InstalledApplicationPublicationOwner {
    pub fn admit_authored_output_retirement(
        &self,
        predecessor: &RequiredOutputPublicationAuthority,
    ) -> Option<AdmittedAuthoredOutputRetirement> {
        Arc::ptr_eq(&self.occurrence, &predecessor.occurrence).then(|| {
            AdmittedAuthoredOutputRetirement {
                predecessors: vec![predecessor.clone()],
            }
        })
    }
    pub fn admit_authored_output_retirements<'a>(
        &self,
        predecessors: impl IntoIterator<Item = &'a RequiredOutputPublicationAuthority>,
    ) -> Option<AdmittedAuthoredOutputRetirement> {
        let predecessors: Vec<_> = predecessors.into_iter().cloned().collect();
        if predecessors.is_empty()
            || predecessors.iter().enumerate().any(|(index, prior)| {
                !Arc::ptr_eq(&self.occurrence, &prior.occurrence)
                    || predecessors[..index].iter().any(|other| {
                        other.publication == prior.publication && other.ordinal == prior.ordinal
                    })
            })
        {
            return None;
        }
        Some(AdmittedAuthoredOutputRetirement { predecessors })
    }
    pub fn admit_required_output_succession(
        &self,
        running: AdmittedRequiredOutputPublication,
        predecessor: &RequiredOutputPublicationAuthority,
    ) -> Option<AdmittedRequiredOutputSuccession> {
        (Arc::ptr_eq(&self.occurrence, &running.occurrence)
            && Arc::ptr_eq(&self.occurrence, &predecessor.occurrence))
        .then(|| AdmittedRequiredOutputSuccession {
            running,
            predecessor: predecessor.clone(),
        })
    }
    pub fn fresh() -> Self {
        Self {
            occurrence: Arc::new(()),
        }
    }

    pub fn authority(&self) -> InstalledApplicationPublicationAuthority {
        InstalledApplicationPublicationAuthority {
            occurrence: Arc::clone(&self.occurrence),
        }
    }

    pub fn admit_checkpoint_program_activation(
        &self,
        commitment: [u8; 32],
    ) -> AdmittedCheckpointProgramActivation {
        AdmittedCheckpointProgramActivation {
            occurrence: Arc::clone(&self.occurrence),
            commitment,
        }
    }

    pub fn admit_required_output(
        &self,
        producer: impl Into<Arc<str>>,
        dependency_identity: [u8; 32],
    ) -> AdmittedRequiredOutputPublication {
        AdmittedRequiredOutputPublication {
            occurrence: Arc::clone(&self.occurrence),
            producer: producer.into(),
            dependency_identity,
        }
    }
}

impl AdmittedAuthoredOutputRetirement {
    pub fn publication_count(&self) -> usize {
        self.predecessors.len()
    }
    pub fn for_publication(
        &self,
        publication: &RequiredOutputPublicationAuthority,
    ) -> Option<Self> {
        self.predecessors
            .iter()
            .any(|prior| {
                Arc::ptr_eq(&prior.occurrence, &publication.occurrence)
                    && prior.publication == publication.publication
                    && prior.ordinal == publication.ordinal
            })
            .then(|| Self {
                predecessors: vec![publication.clone()],
            })
    }
}

impl AdmittedCheckpointProgramActivation {
    pub fn activation_commitment(&self) -> &[u8; 32] {
        &self.commitment
    }
}

impl InstalledApplicationPublicationAuthority {
    pub fn admits_authored_output_retirement(
        &self,
        proof: &AdmittedAuthoredOutputRetirement,
        predecessor: &RequiredOutputPublicationAuthority,
    ) -> bool {
        Arc::ptr_eq(&self.occurrence, &predecessor.occurrence)
            && proof.predecessors.iter().any(|prior| {
                Arc::ptr_eq(&self.occurrence, &prior.occurrence)
                    && predecessor.publication == prior.publication
                    && predecessor.ordinal == prior.ordinal
            })
    }
    pub fn same_publication_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.occurrence, &other.occurrence)
    }

    /// Authenticate the installed issuer of a retained publication seal.
    /// Recorded origin descriptions carry no live issuer identity.
    pub fn owns_required_output_publication(
        &self,
        seal: &RequiredOutputPublicationAuthority,
    ) -> bool {
        Arc::ptr_eq(&self.occurrence, &seal.occurrence)
    }

    pub fn seal_required_output_publication(
        &self,
        publication: u64,
    ) -> RequiredOutputPublicationAuthority {
        self.seal_required_output_publication_at(publication, 0)
    }

    pub fn seal_required_output_publication_at(
        &self,
        publication: u64,
        ordinal: usize,
    ) -> RequiredOutputPublicationAuthority {
        RequiredOutputPublicationAuthority {
            occurrence: Arc::clone(&self.occurrence),
            publication,
            ordinal,
        }
    }

    pub fn admits_required_output_succession(
        &self,
        proof: &AdmittedRequiredOutputSuccession,
        predecessor: &RequiredOutputPublicationAuthority,
    ) -> bool {
        self.admits_required_output(&proof.running)
            && Arc::ptr_eq(&self.occurrence, &predecessor.occurrence)
            && Arc::ptr_eq(&self.occurrence, &proof.predecessor.occurrence)
            && predecessor.publication == proof.predecessor.publication
            && predecessor.ordinal == proof.predecessor.ordinal
    }
    pub fn admits_checkpoint_program_activation(
        &self,
        proof: &AdmittedCheckpointProgramActivation,
    ) -> bool {
        Arc::ptr_eq(&self.occurrence, &proof.occurrence)
    }
    pub fn admits_required_output(&self, proof: &AdmittedRequiredOutputPublication) -> bool {
        Arc::ptr_eq(&self.occurrence, &proof.occurrence)
    }
}

impl AdmittedRequiredOutputSuccession {
    pub fn running(&self) -> &AdmittedRequiredOutputPublication {
        &self.running
    }
}

impl AdmittedRequiredOutputPublication {
    pub fn producer(&self) -> &str {
        &self.producer
    }
    pub fn dependency_identity(&self) -> &[u8; 32] {
        &self.dependency_identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_retirement_seals_distinct_publications_at_the_same_commit() {
        let owner = InstalledApplicationPublicationOwner::fresh();
        let foreign = InstalledApplicationPublicationOwner::fresh();
        let first = owner.authority().seal_required_output_publication_at(9, 0);
        let second = owner.authority().seal_required_output_publication_at(9, 1);
        let stranger = foreign
            .authority()
            .seal_required_output_publication_at(9, 1);
        assert!(owner
            .admit_authored_output_retirements(std::iter::empty())
            .is_none());
        assert!(owner
            .admit_authored_output_retirements([&first, &first])
            .is_none());
        assert!(owner
            .admit_authored_output_retirements([&first, &stranger])
            .is_none());
        let proof = owner
            .admit_authored_output_retirements([&first, &second])
            .unwrap();
        assert_eq!(proof.publication_count(), 2);
        let projected = proof.for_publication(&second).unwrap();
        assert!(owner
            .authority()
            .admits_authored_output_retirement(&projected, &second));
        assert!(!owner
            .authority()
            .admits_authored_output_retirement(&projected, &first));
        let succession = owner
            .admit_required_output_succession(owner.admit_required_output("next", [1; 32]), &first)
            .unwrap();
        assert!(!owner
            .authority()
            .admits_required_output_succession(&succession, &second));
    }

    #[test]
    fn succession_requires_original_installed_owner_and_exact_predecessor_publication() {
        let owner = InstalledApplicationPublicationOwner::fresh();
        let other = InstalledApplicationPublicationOwner::fresh();
        let first = owner.authority().seal_required_output_publication(11);
        let second = owner.authority().seal_required_output_publication(12);
        let foreign = other.authority().seal_required_output_publication(11);
        assert!(owner
            .admit_required_output_succession(
                owner.admit_required_output("preserver", [2; 32]),
                &foreign
            )
            .is_none());
        let proof = owner
            .admit_required_output_succession(
                owner.admit_required_output("preserver", [2; 32]),
                &first,
            )
            .unwrap();
        assert!(owner
            .authority()
            .admits_required_output_succession(&proof, &first));
        assert!(!owner
            .authority()
            .admits_required_output_succession(&proof, &second));
        assert!(!other
            .authority()
            .admits_required_output_succession(&proof, &foreign));
    }

    #[test]
    fn equal_producer_text_does_not_replace_installed_owner() {
        let installed = InstalledApplicationPublicationOwner::fresh();
        let foreign = InstalledApplicationPublicationOwner::fresh();
        let proof = foreign.admit_required_output("producer", [7; 32]);
        assert!(!installed.authority().admits_required_output(&proof));
        let genuine = installed.admit_required_output("producer", [7; 32]);
        assert!(installed.authority().admits_required_output(&genuine));
    }

    #[test]
    fn publication_seals_require_the_installed_issuer_even_at_equal_coordinates() {
        let owner = InstalledApplicationPublicationOwner::fresh();
        let foreign = InstalledApplicationPublicationOwner::fresh();
        let own = owner.authority().seal_required_output_publication_at(9, 1);
        let other = foreign
            .authority()
            .seal_required_output_publication_at(9, 1);
        assert!(owner.authority().owns_required_output_publication(&own));
        assert!(!owner.authority().owns_required_output_publication(&other));
        assert!(!foreign.authority().owns_required_output_publication(&own));
    }

    #[test]
    fn activation_admission_retains_exact_commitment_and_installed_owner() {
        let installed = InstalledApplicationPublicationOwner::fresh();
        let foreign = InstalledApplicationPublicationOwner::fresh();
        let proof = installed.admit_checkpoint_program_activation([19; 32]);
        assert_eq!(proof.activation_commitment(), &[19; 32]);
        assert!(installed
            .authority()
            .admits_checkpoint_program_activation(&proof));
        assert!(!foreign
            .authority()
            .admits_checkpoint_program_activation(&proof));
    }
}
