//! Query readmission stays bound to the original Home adapter occurrence.
use crate::{HomeIncarnationDenial, HomeIncarnationFence};
use std::sync::Arc;
mod publication;
pub use publication::{
    CommittedHomeQueryCorrection, HomeQueryCorrectionSubject, NoEffectHomeQueryCorrection,
};

#[derive(Debug)]
pub struct HomeQueryCorrectionsOwner {
    authority: HomeQueryCorrectionsAuthority,
}
#[derive(Clone, Debug)]
pub struct HomeQueryCorrectionsAuthority {
    occurrence: Arc<()>,
    home: [u8; 16],
    incarnation: HomeIncarnationFence,
}
#[derive(Clone, Debug)]
pub struct AdmittedHomeQueryCorrections {
    authority: HomeQueryCorrectionsAuthority,
    position: u64,
    digest: [u8; 32],
    payload_digest: [u8; 32],
}
impl HomeQueryCorrectionsOwner {
    pub fn fresh(home: [u8; 16], incarnation: HomeIncarnationFence) -> Self {
        Self {
            authority: HomeQueryCorrectionsAuthority {
                occurrence: Arc::new(()),
                home,
                incarnation,
            },
        }
    }
    pub fn authority(&self) -> HomeQueryCorrectionsAuthority {
        self.authority.clone()
    }
    pub fn admit(&self, proof: &AdmittedHomeQueryCorrections) -> Result<(), HomeIncarnationDenial> {
        self.authority.admit(&proof.authority)
    }
}
impl HomeQueryCorrectionsAuthority {
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
    pub fn admit_query_correction_prefix(
        &self,
        position: u64,
        digest: [u8; 32],
        payload_digest: [u8; 32],
    ) -> AdmittedHomeQueryCorrections {
        AdmittedHomeQueryCorrections {
            authority: self.clone(),
            position,
            digest,
            payload_digest,
        }
    }
}
impl AdmittedHomeQueryCorrections {
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
