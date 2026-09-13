use crate::BlobResumeReplayReadmission;
use worth_store_authority::StoreCurrentAuthorityWitness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobResumeStoreAuthority {
    authority_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobResumeReadmissionAuthority {
    authority_digest: String,
    replay_readmission: BlobResumeReplayReadmission,
}

impl BlobResumeStoreAuthority {
    pub fn from_current_store_authority(authority: StoreCurrentAuthorityWitness) -> Self {
        Self {
            authority_digest: authority_digest(&authority),
        }
    }

    pub fn authority_digest(&self) -> &str {
        &self.authority_digest
    }
}

impl BlobResumeReadmissionAuthority {
    pub fn from_recovery_readmission(readmission: BlobResumeReplayReadmission) -> Self {
        Self {
            authority_digest: readmission.current_store_authority_digest().to_owned(),
            replay_readmission: readmission,
        }
    }

    pub fn authority_digest(&self) -> &str {
        &self.authority_digest
    }

    pub fn replay_checkpoint_source_digest(&self) -> &str {
        self.replay_readmission.checkpoint_source_digest()
    }
}

pub(crate) fn authority_digest(authority: &StoreCurrentAuthorityWitness) -> String {
    authority.identity().aspect_key().as_str().to_owned()
}
