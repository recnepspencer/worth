//! The installed program owner attests one exact selected branch source.
use std::sync::Arc;
#[derive(Debug)]
pub struct InstalledBranchProgramSourceOwner {
    occurrence: Arc<()>,
}
#[derive(Clone, Debug)]
pub struct InstalledBranchProgramSourceAuthority {
    occurrence: Arc<()>,
}
#[derive(Debug)]
pub struct AdmittedBranchProgramSource {
    occurrence: Arc<()>,
    revision: [u8; 32],
    commitment: [u8; 32],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchProgramSourceAdmissionDenial {
    ForeignOwner,
    SourceMismatch,
}
impl InstalledBranchProgramSourceOwner {
    pub fn fresh() -> Self {
        Self {
            occurrence: Arc::new(()),
        }
    }
    pub fn authority(&self) -> InstalledBranchProgramSourceAuthority {
        InstalledBranchProgramSourceAuthority {
            occurrence: Arc::clone(&self.occurrence),
        }
    }
    pub fn admit_source(
        &self,
        revision: [u8; 32],
        commitment: [u8; 32],
    ) -> AdmittedBranchProgramSource {
        AdmittedBranchProgramSource {
            occurrence: Arc::clone(&self.occurrence),
            revision,
            commitment,
        }
    }
}
impl AdmittedBranchProgramSource {
    pub fn program_revision(&self) -> [u8; 32] {
        self.revision
    }
}
impl InstalledBranchProgramSourceAuthority {
    pub fn admit(
        &self,
        source: &AdmittedBranchProgramSource,
        commitment: &[u8; 32],
    ) -> Result<(), BranchProgramSourceAdmissionDenial> {
        if !Arc::ptr_eq(&self.occurrence, &source.occurrence) {
            return Err(BranchProgramSourceAdmissionDenial::ForeignOwner);
        }
        if &source.commitment != commitment {
            return Err(BranchProgramSourceAdmissionDenial::SourceMismatch);
        }
        Ok(())
    }
}
