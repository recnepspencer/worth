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

/// Admission issued only after the installed owner authenticates its recovered
/// program activation predecessor. Native adoption still binds the exact patch.
#[derive(Debug)]
pub struct AdmittedCheckpointProgramActivation {
    occurrence: Arc<()>,
    commitment: [u8; 32],
}

impl InstalledApplicationPublicationOwner {
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
