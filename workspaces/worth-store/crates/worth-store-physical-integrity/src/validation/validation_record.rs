use worth_store_physical_format::integrity_declarations::{
    PhysicalIntegrityAlgorithm, PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

use super::PhysicalArtifactScope;

/// Byte-free validation description owned by a runtime lifecycle entry.
///
/// This record deliberately retains the exact scope rather than using a lossy
/// process hash as the correctness key. It grants no access to inspected bytes
/// and cannot open an owner decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalIntegrityValidationRecord {
    scope: PhysicalArtifactScope,
    exact_scope_digest: PhysicalIntegrityValidationDigest,
    byte_range_digest: PhysicalIntegrityValidationDigest,
    mechanism: PhysicalIntegrityValidationMechanism,
}

impl PhysicalIntegrityValidationRecord {
    pub(crate) fn from_validated_scope(
        scope: PhysicalArtifactScope,
        exact_scope_digest: PhysicalIntegrityValidationDigest,
        byte_range_digest: PhysicalIntegrityValidationDigest,
        mechanism: PhysicalIntegrityValidationMechanism,
    ) -> Option<Self> {
        let declaration = scope.declaration();
        let algorithm = mechanism.algorithm();
        if exact_scope_digest.algorithm() != algorithm
            || byte_range_digest.algorithm() != algorithm
            || declaration.checksums().is_empty()
            || !declaration
                .checksums()
                .iter()
                .all(|checksum| checksum.algorithm() == algorithm)
        {
            return None;
        }
        Some(Self {
            scope,
            exact_scope_digest,
            byte_range_digest,
            mechanism,
        })
    }

    pub const fn artifact_family(self) -> PhysicalIntegrityArtifactFamily {
        self.scope.artifact_family()
    }

    pub const fn declaration(self) -> PhysicalIntegrityFormatDeclaration {
        self.scope.declaration()
    }

    /// Digest of the expected Store/family/artifact/generation/range scope.
    pub const fn exact_scope_digest(self) -> PhysicalIntegrityValidationDigest {
        self.exact_scope_digest
    }

    /// Digest of the exact bytes inspected under that scope.
    pub const fn byte_range_digest(self) -> PhysicalIntegrityValidationDigest {
        self.byte_range_digest
    }

    pub const fn mechanism(self) -> PhysicalIntegrityValidationMechanism {
        self.mechanism
    }

    pub fn matches_scope(self, scope: PhysicalArtifactScope) -> bool {
        self.scope == scope
    }

    #[cfg(feature = "test-support")]
    pub fn for_test(
        scope: PhysicalArtifactScope,
        exact_scope_digest: PhysicalIntegrityValidationDigest,
        byte_range_digest: PhysicalIntegrityValidationDigest,
        mechanism: PhysicalIntegrityValidationMechanism,
    ) -> Self {
        Self::from_validated_scope(scope, exact_scope_digest, byte_range_digest, mechanism)
            .expect("test validation evidence must match the scope declaration")
    }
}

/// Algorithm-tagged digest retained by the byte-free C.6 validation record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityValidationDigest {
    Crc32c(u32),
    Sha256([u8; 32]),
}

impl PhysicalIntegrityValidationDigest {
    pub const fn crc32c(value: u32) -> Self {
        Self::Crc32c(value)
    }

    pub const fn sha256(value: [u8; 32]) -> Self {
        Self::Sha256(value)
    }

    pub const fn algorithm(self) -> PhysicalIntegrityAlgorithm {
        match self {
            Self::Crc32c(_) => PhysicalIntegrityAlgorithm::Crc32c,
            Self::Sha256(_) => PhysicalIntegrityAlgorithm::Sha256,
        }
    }
}

/// Frozen validator mechanism/version retained with successful evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityValidationMechanism {
    Crc32cV1,
    Sha256V1,
}

impl PhysicalIntegrityValidationMechanism {
    pub const fn algorithm(self) -> PhysicalIntegrityAlgorithm {
        match self {
            Self::Crc32cV1 => PhysicalIntegrityAlgorithm::Crc32c,
            Self::Sha256V1 => PhysicalIntegrityAlgorithm::Sha256,
        }
    }

    pub const fn version(self) -> u16 {
        1
    }
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::integrity_declarations::{
        families::root::CURRENT_SELECTOR_INTEGRITY_DECLARATION, PhysicalIntegrityAlgorithm,
    };
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::{
        PhysicalIntegrityValidationDigest as Digest,
        PhysicalIntegrityValidationMechanism as Mechanism, PhysicalIntegrityValidationRecord,
    };
    use crate::{PhysicalArtifactScope, PhysicalByteRange};

    fn store(byte: u8) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
        )
        .published_identity()
    }

    fn selector_scope(byte: u8, length: u64) -> PhysicalArtifactScope {
        PhysicalArtifactScope::current_root_selector(
            store(byte),
            worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
                .admit()
                .unwrap(),
            PhysicalByteRange::new(0, length).unwrap(),
        )
    }

    #[test]
    fn record_retain_crc32c_family_scope_range_and_mechanism_evidence() {
        let scope = selector_scope(7, 107);
        let record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            Digest::crc32c(11),
            Digest::crc32c(12),
            Mechanism::Crc32cV1,
        )
        .unwrap();

        assert_eq!(
            record.artifact_family(),
            CURRENT_SELECTOR_INTEGRITY_DECLARATION.family()
        );
        assert_eq!(record.exact_scope_digest(), Digest::crc32c(11));
        assert_eq!(record.byte_range_digest(), Digest::crc32c(12));
        assert_eq!(record.mechanism().version(), 1);
    }

    #[test]
    fn record_cannot_be_retargeted_across_store_or_range() {
        let scope = selector_scope(8, 107);
        let record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            Digest::crc32c(21),
            Digest::crc32c(22),
            Mechanism::Crc32cV1,
        )
        .unwrap();

        assert!(record.matches_scope(scope));
        assert!(!record.matches_scope(selector_scope(9, 107)));
        assert!(!record.matches_scope(selector_scope(8, 106)));
        assert_eq!(
            record.mechanism().algorithm(),
            PhysicalIntegrityAlgorithm::Crc32c
        );
    }

    #[test]
    fn record_cannot_be_retargeted_across_root_generation() {
        let range = PhysicalByteRange::new(0, 107).unwrap();
        let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap();
        let scope = PhysicalArtifactScope::root_manifest(store(11), format, 41, range).unwrap();
        let record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            Digest::crc32c(31),
            Digest::crc32c(32),
            Mechanism::Crc32cV1,
        )
        .unwrap();

        assert!(record.matches_scope(scope));
        assert!(!record.matches_scope(
            PhysicalArtifactScope::root_manifest(store(11), format, 42, range).unwrap()
        ));
    }

    #[test]
    fn record_rejects_digest_or_mechanism_substitution() {
        let scope = selector_scope(10, 107);
        assert!(PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            Digest::sha256([1; 32]),
            Digest::crc32c(2),
            Mechanism::Sha256V1,
        )
        .is_none());
        assert!(PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            Digest::sha256([1; 32]),
            Digest::sha256([2; 32]),
            Mechanism::Sha256V1,
        )
        .is_none());
    }
}
