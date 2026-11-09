//! Terminal permission belongs to the original adapter and exact frame.
use super::*;
use crate::{AuthorityMarker, DurablyCommitted};

#[derive(Debug)]
pub struct HomeQueryCorrectionSubject {
    authority: HomeQueryCorrectionsAuthority,
    position: u64,
    payload_digest: [u8; 32],
}
impl HomeQueryCorrectionSubject {
    pub fn admit_to(
        &self,
        receiver: &HomeQueryCorrectionsAuthority,
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
pub struct CommittedHomeQueryCorrection {
    subject: HomeQueryCorrectionSubject,
}
impl AuthorityMarker for CommittedHomeQueryCorrection {}
impl CommittedHomeQueryCorrection {
    pub const fn subject(&self) -> &HomeQueryCorrectionSubject {
        &self.subject
    }
}

/// No-effect custody cannot authorize a committed cutover.
#[derive(Debug)]
pub struct NoEffectHomeQueryCorrection {
    subject: HomeQueryCorrectionSubject,
}
impl NoEffectHomeQueryCorrection {
    pub const fn subject(&self) -> &HomeQueryCorrectionSubject {
        &self.subject
    }
}

impl HomeQueryCorrectionsOwner {
    /// The Store binding calls this only while retaining actual committed custody.
    pub fn acknowledge_query_correction_commit(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> DurablyCommitted<CommittedHomeQueryCorrection> {
        DurablyCommitted::from_owner(CommittedHomeQueryCorrection {
            subject: self.publication_subject(position, payload_digest),
        })
    }
    /// The original publication was proved unstarted or physically no-effect.
    pub fn acknowledge_query_correction_no_effect(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> NoEffectHomeQueryCorrection {
        NoEffectHomeQueryCorrection {
            subject: self.publication_subject(position, payload_digest),
        }
    }
    fn publication_subject(
        &self,
        position: u64,
        payload_digest: [u8; 32],
    ) -> HomeQueryCorrectionSubject {
        HomeQueryCorrectionSubject {
            authority: self.authority(),
            position,
            payload_digest,
        }
    }
}
