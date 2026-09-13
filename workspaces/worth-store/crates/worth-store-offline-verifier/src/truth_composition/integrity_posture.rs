#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityPosture {
    Confirmed,
    DigestMismatch,
    IntegrityNotDeclared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineIntegrityObservation {
    posture: OfflineIntegrityPosture,
    observed_digest: [u8; 32],
    expected_digest: Option<[u8; 32]>,
}

impl OfflineIntegrityObservation {
    pub const fn posture(self) -> OfflineIntegrityPosture {
        self.posture
    }

    pub const fn observed_digest(self) -> [u8; 32] {
        self.observed_digest
    }

    pub const fn expected_digest(self) -> Option<[u8; 32]> {
        self.expected_digest
    }
}

pub(super) fn classify_offline_integrity(
    observed_digest: [u8; 32],
    expected_digest: Option<[u8; 32]>,
) -> OfflineIntegrityObservation {
    let posture = match expected_digest {
        Some(expected) if expected == observed_digest => OfflineIntegrityPosture::Confirmed,
        Some(_) => OfflineIntegrityPosture::DigestMismatch,
        None => OfflineIntegrityPosture::IntegrityNotDeclared,
    };
    OfflineIntegrityObservation {
        posture,
        observed_digest,
        expected_digest,
    }
}
