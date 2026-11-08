//! Installed composite affinity admits a single owner-defined Signal binding.
use std::sync::Arc;

/// The composing runtime retains this issuer beside its original open custody.
#[derive(Debug)]
pub struct InstalledCompositeSignalPublicationOwner {
    occurrence: Arc<()>,
}

#[derive(Clone, Debug)]
pub struct InstalledCompositeSignalPublicationAuthority {
    occurrence: Arc<()>,
}

/// Move-only legality; the Signal owner separately validates the bound meaning.
#[derive(Debug)]
pub struct AdmittedCompositeSignalPublication {
    occurrence: Arc<()>,
    commitment: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeSignalPublicationAdmissionDenial {
    ForeignOwner,
    BindingMismatch,
}

impl InstalledCompositeSignalPublicationOwner {
    pub fn fresh() -> Self {
        Self {
            occurrence: Arc::new(()),
        }
    }

    pub fn authority(&self) -> InstalledCompositeSignalPublicationAuthority {
        InstalledCompositeSignalPublicationAuthority {
            occurrence: Arc::clone(&self.occurrence),
        }
    }

    pub fn admit_original(&self, commitment: [u8; 32]) -> AdmittedCompositeSignalPublication {
        AdmittedCompositeSignalPublication {
            occurrence: Arc::clone(&self.occurrence),
            commitment,
        }
    }
}

impl InstalledCompositeSignalPublicationAuthority {
    pub fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.occurrence, &other.occurrence)
    }

    pub fn admit(
        &self,
        grant: &AdmittedCompositeSignalPublication,
        commitment: &[u8; 32],
    ) -> Result<(), CompositeSignalPublicationAdmissionDenial> {
        if !Arc::ptr_eq(&self.occurrence, &grant.occurrence) {
            return Err(CompositeSignalPublicationAdmissionDenial::ForeignOwner);
        }
        if &grant.commitment != commitment {
            return Err(CompositeSignalPublicationAdmissionDenial::BindingMismatch);
        }
        Ok(())
    }
}
