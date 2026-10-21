use std::sync::Arc;

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
}

#[derive(Debug)]
pub struct AdmittedRequiredOutputSuccession {
    running: AdmittedRequiredOutputPublication,
    predecessor: RequiredOutputPublicationAuthority,
}

/// Admission issued only after the installed owner authenticates its recovered
/// program activation predecessor. Native adoption still binds the exact patch.
#[derive(Debug)]
pub struct AdmittedCheckpointProgramActivation {
    occurrence: Arc<()>,
    commitment: [u8; 32],
}

impl InstalledApplicationPublicationOwner {
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

impl AdmittedCheckpointProgramActivation {
    pub fn activation_commitment(&self) -> &[u8; 32] {
        &self.commitment
    }
}

impl InstalledApplicationPublicationAuthority {
    pub fn same_publication_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.occurrence, &other.occurrence)
    }

    pub fn seal_required_output_publication(
        &self,
        publication: u64,
    ) -> RequiredOutputPublicationAuthority {
        RequiredOutputPublicationAuthority {
            occurrence: Arc::clone(&self.occurrence),
            publication,
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
