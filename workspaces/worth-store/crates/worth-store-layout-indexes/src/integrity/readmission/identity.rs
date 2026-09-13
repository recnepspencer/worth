use super::authority::RecoveryLayoutReadmissionIdentity;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutReadmissionIdentity([u8; 32]);

impl LayoutReadmissionIdentity {
    pub(super) fn from_recovery(identity: &RecoveryLayoutReadmissionIdentity) -> Self {
        let mut digest = Sha256::new();
        match identity {
            RecoveryLayoutReadmissionIdentity::QuarantineObservation(observation) => {
                digest.update(b"quarantine-observation");
                update_field(&mut digest, observation.as_str());
            }
        }
        Self(digest.finalize().into())
    }

    pub const fn fingerprint(self) -> [u8; 32] {
        self.0
    }
}

fn update_field(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value.as_bytes());
}
