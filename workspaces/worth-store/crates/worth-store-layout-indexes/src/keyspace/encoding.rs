use super::blob_identity::blob_identity_digest_bytes;
use super::composite::{declare_composite_key_ordering, CompositeKeyOrderingLaw};
use super::declaration::{PhysicalKeyDomain, PhysicalKeyDomainWitness};
use super::value::ConcretePhysicalKeyWitness;
use crate::catalog::ArtifactFamilyDenial;
use crate::BlobIdentityKeyBasis;
use worth_store_contracts::WalRecordFamily;
use worth_store_physical_format::PhysicalReferenceKind;
use worth_store_security::{StoreKeyScope, StoreTenantScope};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingSentinelPolicy {
    NoSentinel,
    PrefixSuccessorByte,
    RangeExclusiveEndByte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalKeyEncoding {
    domain: PhysicalKeyDomainWitness,
    version_byte: u8,
    composite_order: CompositeKeyOrderingLaw,
    sentinel_policy: EncodingSentinelPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalKeyBytes {
    encoding: CanonicalKeyEncoding,
    bytes: Vec<u8>,
}

impl CanonicalKeyBytes {
    pub(crate) fn new(encoding: CanonicalKeyEncoding, bytes: Vec<u8>) -> Self {
        Self { encoding, bytes }
    }

    pub const fn encoding(&self) -> CanonicalKeyEncoding {
        self.encoding
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl CanonicalKeyEncoding {
    pub(crate) const fn new(
        domain: PhysicalKeyDomainWitness,
        version_byte: u8,
        composite_order: CompositeKeyOrderingLaw,
        sentinel_policy: EncodingSentinelPolicy,
    ) -> Self {
        Self {
            domain,
            version_byte,
            composite_order,
            sentinel_policy,
        }
    }

    pub const fn domain(self) -> PhysicalKeyDomainWitness {
        self.domain
    }

    pub const fn version_byte(self) -> u8 {
        self.version_byte
    }

    pub const fn composite_order(self) -> CompositeKeyOrderingLaw {
        self.composite_order
    }

    pub const fn sentinel_policy(self) -> EncodingSentinelPolicy {
        self.sentinel_policy
    }
}

pub(crate) const fn require_canonical_key_encoding(
    domain: PhysicalKeyDomainWitness,
) -> CanonicalKeyEncoding {
    let version_byte = match domain.domain() {
        PhysicalKeyDomain::RootManifestKey => 0x10,
        PhysicalKeyDomain::PageAddressKey => 0x20,
        PhysicalKeyDomain::SegmentAddressKey => 0x21,
        PhysicalKeyDomain::ExtentAddressKey => 0x22,
        PhysicalKeyDomain::PhysicalReferenceKey => 0x23,
        PhysicalKeyDomain::WalRecordKey => 0x30,
        PhysicalKeyDomain::BlobIdentityKey => 0x40,
    };
    let sentinel_policy = match domain.domain() {
        PhysicalKeyDomain::RootManifestKey => EncodingSentinelPolicy::NoSentinel,
        PhysicalKeyDomain::PageAddressKey
        | PhysicalKeyDomain::SegmentAddressKey
        | PhysicalKeyDomain::ExtentAddressKey
        | PhysicalKeyDomain::BlobIdentityKey
        | PhysicalKeyDomain::PhysicalReferenceKey => EncodingSentinelPolicy::PrefixSuccessorByte,
        PhysicalKeyDomain::WalRecordKey => EncodingSentinelPolicy::RangeExclusiveEndByte,
    };

    CanonicalKeyEncoding::new(
        domain,
        version_byte,
        declare_composite_key_ordering(domain),
        sentinel_policy,
    )
}

pub(crate) fn encode_concrete_physical_key(
    encoding: CanonicalKeyEncoding,
    key: ConcretePhysicalKeyWitness,
) -> Result<CanonicalKeyBytes, ArtifactFamilyDenial> {
    if encoding.domain() != key.domain() {
        return Err(ArtifactFamilyDenial::ConcreteKeyKindDoesNotMatchPhysicalKeyDomain);
    }

    let mut bytes = encode_scope_prefix(encoding, &key);

    if let Some(root_reference) = key.root_reference() {
        bytes.extend_from_slice(&root_reference.get().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some((segment_id, page_id)) = key.page_address() {
        bytes.extend_from_slice(&segment_id.get().to_be_bytes());
        bytes.extend_from_slice(&page_id.get().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some(segment_id) = key.segment_id()
        && key.extent_address().is_none()
        && key.page_address().is_none()
    {
        bytes.extend_from_slice(&segment_id.get().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some((segment_id, extent_id)) = key.extent_address() {
        bytes.extend_from_slice(&segment_id.get().to_be_bytes());
        bytes.extend_from_slice(&extent_id.get().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some(reference) = key.physical_reference() {
        bytes.push(physical_reference_kind_code(reference.kind()));
        bytes.extend_from_slice(
            &reference
                .segment_id()
                .map(worth_store_physical_format::PhysicalSegmentId::get)
                .unwrap_or_default()
                .to_be_bytes(),
        );
        bytes.extend_from_slice(
            &reference
                .page_id()
                .map(worth_store_physical_format::PhysicalPageId::get)
                .unwrap_or_default()
                .to_be_bytes(),
        );
        bytes.extend_from_slice(
            &reference
                .extent_id()
                .map(worth_store_physical_format::PhysicalExtentId::get)
                .unwrap_or_default()
                .to_be_bytes(),
        );
        bytes.extend_from_slice(
            &u64::from(
                reference
                    .slot()
                    .map(worth_store_physical_format::PhysicalRecordSlot::get)
                    .unwrap_or_default(),
            )
            .to_be_bytes(),
        );
        bytes.extend_from_slice(&reference.generation().get().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some((family, record_identity)) = key.wal_record() {
        bytes.push(wal_record_family_code(family));
        bytes.extend_from_slice(&record_identity.sequence().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    if let Some(identity) = key.blob_identity() {
        bytes.extend_from_slice(blob_identity_digest_bytes(identity));
        bytes.extend_from_slice(&identity.generation().sequence().to_be_bytes());
        return Ok(CanonicalKeyBytes::new(encoding, bytes));
    }

    Err(ArtifactFamilyDenial::ConcreteKeyKindDoesNotMatchPhysicalKeyDomain)
}

pub(crate) fn encode_scope_prefix(
    encoding: CanonicalKeyEncoding,
    key: &ConcretePhysicalKeyWitness,
) -> Vec<u8> {
    let mut bytes = vec![encoding.version_byte()];
    push_tenant_scope(&mut bytes, key.tenant_scope());
    push_key_scope(&mut bytes, key.key_scope());
    bytes
}

pub(crate) fn encode_blob_identity_prefix(
    encoding: CanonicalKeyEncoding,
    identity: &BlobIdentityKeyBasis,
) -> CanonicalKeyBytes {
    let mut bytes = vec![encoding.version_byte()];
    push_tenant_scope(
        &mut bytes,
        encoding.domain().scope().admitted_tenant_scope(),
    );
    push_key_scope(&mut bytes, encoding.domain().scope().admitted_key_scope());
    bytes.extend_from_slice(blob_identity_digest_bytes(identity));
    CanonicalKeyBytes::new(encoding, bytes)
}

pub(crate) fn exclusive_bound_sentinel(encoding: CanonicalKeyEncoding) -> u8 {
    match encoding.sentinel_policy() {
        EncodingSentinelPolicy::NoSentinel => 0,
        EncodingSentinelPolicy::PrefixSuccessorByte => 0xFF,
        EncodingSentinelPolicy::RangeExclusiveEndByte => 0xFE,
    }
}

pub(crate) fn push_tenant_scope(bytes: &mut Vec<u8>, scope: StoreTenantScope) {
    bytes.push(match scope {
        StoreTenantScope::StoreInternal => 0x01,
        StoreTenantScope::TenantPhysicalBoundary => 0x02,
        StoreTenantScope::MultiTenantPhysicalBoundary => 0x03,
        StoreTenantScope::RepairBlastRadius => 0x04,
        StoreTenantScope::ImportReadmissionBoundary => 0x05,
        StoreTenantScope::BackupRestoreBoundary => 0x06,
        StoreTenantScope::SecurityLifecycleFoundation => 0x07,
    });
}

pub(crate) fn push_key_scope(bytes: &mut Vec<u8>, scope: StoreKeyScope) {
    bytes.push(match scope {
        StoreKeyScope::StoreManagedRoot => 0x01,
        StoreKeyScope::TenantEnvelope => 0x02,
        StoreKeyScope::ArtifactEnvelope => 0x03,
        StoreKeyScope::PageEnvelope => 0x04,
        StoreKeyScope::WalCheckpointEnvelope => 0x05,
        StoreKeyScope::BlobChunkEnvelope => 0x06,
        StoreKeyScope::BackupExportEnvelope => 0x07,
        StoreKeyScope::RepairScopeEnvelope => 0x08,
        StoreKeyScope::SecurityLifecycleFoundation => 0x09,
    });
}

fn wal_record_family_code(family: WalRecordFamily) -> u8 {
    match family {
        WalRecordFamily::DurableMutationIntent => 0x01,
        WalRecordFamily::HostedRuntimeCommitResult => 0x02,
        WalRecordFamily::BulkCheckpointPublicationIntent => 0x03,
        WalRecordFamily::DurablePublicationProgress => 0x04,
        WalRecordFamily::RecoveryDecision => 0x05,
    }
}

pub(crate) fn physical_reference_kind_code(kind: PhysicalReferenceKind) -> u8 {
    match kind {
        PhysicalReferenceKind::PageSlot => 0x01,
        PhysicalReferenceKind::ExtentBacked => 0x02,
        PhysicalReferenceKind::FreeSpaceReuse => 0x03,
        PhysicalReferenceKind::RootPublication => 0x04,
    }
}
