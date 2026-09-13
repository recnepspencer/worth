use super::identity::{decode_identity, encode_identity};
use super::record::{read_u64, CheckpointStreamDecodeDenial};
use super::PhysicalCheckpointIdentity;
use sha2::{Digest, Sha256};

pub(super) const HEADER_PAYLOAD_BYTES: usize = 144;
pub const CHECKPOINT_STREAM_HEADER_RECORD_BYTES: usize = 164;
const CONCURRENT_MUTATION_POSTURE: u8 = 1;
const SECURITY_BINDING_PRESENT: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointWalSourceRange {
    admitted_begin_lsn: u64,
    covered_end_lsn_exclusive: u64,
}

impl CheckpointWalSourceRange {
    pub const fn new(admitted_begin_lsn: u64, covered_end_lsn_exclusive: u64) -> Option<Self> {
        if admitted_begin_lsn >= covered_end_lsn_exclusive {
            return None;
        }
        Some(Self {
            admitted_begin_lsn,
            covered_end_lsn_exclusive,
        })
    }

    pub const fn admitted_begin_lsn(self) -> u64 {
        self.admitted_begin_lsn
    }

    pub const fn covered_end_lsn_exclusive(self) -> u64 {
        self.covered_end_lsn_exclusive
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointRootBasis {
    generation: u64,
    tree_identity: u64,
}

impl CheckpointRootBasis {
    pub const fn new(generation: u64, tree_identity: u64) -> Self {
        Self {
            generation,
            tree_identity,
        }
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn tree_identity(self) -> u64 {
        self.tree_identity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCheckpointSource {
    identity: PhysicalCheckpointIdentity,
    wal: CheckpointWalSourceRange,
    root: CheckpointRootBasis,
    dirty_generation_frontier: u64,
    security_binding: Option<PhysicalCheckpointSecurityBinding>,
}

/// Persisted Store policy binding required to interpret C.7 operation leases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCheckpointSecurityBinding {
    policy_identity: [u8; 32],
    idempotency_retention_generations: u64,
    digest: [u8; 32],
}

impl PhysicalCheckpointSource {
    pub fn decode_stream_header_record(
        record: &[u8],
    ) -> Result<Self, CheckpointStreamDecodeDenial> {
        let payload =
            super::record::decode_record(record, super::record::HEADER_KIND, HEADER_PAYLOAD_BYTES)?;
        decode_header(payload)
    }

    pub const fn concurrent(
        identity: PhysicalCheckpointIdentity,
        wal: CheckpointWalSourceRange,
        root: CheckpointRootBasis,
        dirty_generation_frontier: u64,
    ) -> Self {
        Self {
            identity,
            wal,
            root,
            dirty_generation_frontier,
            security_binding: None,
        }
    }

    pub fn secured_concurrent(
        identity: PhysicalCheckpointIdentity,
        wal: CheckpointWalSourceRange,
        root: CheckpointRootBasis,
        dirty_generation_frontier: u64,
        policy_identity: [u8; 32],
        idempotency_retention_generations: u64,
    ) -> Option<Self> {
        let security_binding = PhysicalCheckpointSecurityBinding::new(
            identity,
            wal,
            root,
            policy_identity,
            idempotency_retention_generations,
        )?;
        Some(Self {
            identity,
            wal,
            root,
            dirty_generation_frontier,
            security_binding: Some(security_binding),
        })
    }

    pub const fn identity(self) -> PhysicalCheckpointIdentity {
        self.identity
    }

    pub const fn wal(self) -> CheckpointWalSourceRange {
        self.wal
    }

    pub const fn root(self) -> CheckpointRootBasis {
        self.root
    }

    pub const fn dirty_generation_frontier(self) -> u64 {
        self.dirty_generation_frontier
    }

    pub const fn security_binding(self) -> Option<PhysicalCheckpointSecurityBinding> {
        self.security_binding
    }
}

impl PhysicalCheckpointSecurityBinding {
    fn new(
        identity: PhysicalCheckpointIdentity,
        wal: CheckpointWalSourceRange,
        root: CheckpointRootBasis,
        policy_identity: [u8; 32],
        idempotency_retention_generations: u64,
    ) -> Option<Self> {
        if policy_identity == [0; 32] || idempotency_retention_generations == 0 {
            return None;
        }
        let digest = security_digest(
            identity,
            wal,
            root,
            policy_identity,
            idempotency_retention_generations,
        );
        Some(Self {
            policy_identity,
            idempotency_retention_generations,
            digest,
        })
    }

    pub const fn policy_identity(self) -> [u8; 32] {
        self.policy_identity
    }

    pub const fn idempotency_retention_generations(self) -> u64 {
        self.idempotency_retention_generations
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

pub(super) fn encode_header(source: PhysicalCheckpointSource) -> [u8; HEADER_PAYLOAD_BYTES] {
    let mut payload = [0; HEADER_PAYLOAD_BYTES];
    encode_identity(&mut payload[..24], source.identity());
    payload[24..32].copy_from_slice(&source.wal().admitted_begin_lsn().to_le_bytes());
    payload[32..40].copy_from_slice(&source.wal().covered_end_lsn_exclusive().to_le_bytes());
    payload[40..48].copy_from_slice(&source.root().generation().to_le_bytes());
    payload[48..56].copy_from_slice(&source.root().tree_identity().to_le_bytes());
    payload[56..64].copy_from_slice(&source.dirty_generation_frontier().to_le_bytes());
    payload[64] = CONCURRENT_MUTATION_POSTURE;
    if let Some(security) = source.security_binding() {
        payload[65] = SECURITY_BINDING_PRESENT;
        payload[72..104].copy_from_slice(&security.policy_identity());
        payload[104..112]
            .copy_from_slice(&security.idempotency_retention_generations().to_le_bytes());
        payload[112..144].copy_from_slice(&security.digest());
    }
    payload
}

pub(super) fn decode_header(
    payload: &[u8],
) -> Result<PhysicalCheckpointSource, CheckpointStreamDecodeDenial> {
    if payload[66..72] != [0; 6] {
        return Err(CheckpointStreamDecodeDenial::ReservedFieldNonZero);
    }
    let identity = decode_identity(&payload[..24])?;
    let wal = CheckpointWalSourceRange::new(read_u64(payload, 24), read_u64(payload, 32))
        .ok_or(CheckpointStreamDecodeDenial::InvalidWalRange)?;
    let root = CheckpointRootBasis::new(read_u64(payload, 40), read_u64(payload, 48));
    if payload[64] != CONCURRENT_MUTATION_POSTURE {
        return Err(CheckpointStreamDecodeDenial::InvalidCapturePosture(
            payload[64],
        ));
    }
    match payload[65] {
        0 => {
            if let Some(index) = payload[72..144].iter().position(|byte| *byte != 0) {
                return Err(CheckpointStreamDecodeDenial::AbsentSecurityBindingResidue {
                    payload_offset: (72 + index) as u8,
                });
            }
            Ok(PhysicalCheckpointSource::concurrent(
                identity,
                wal,
                root,
                read_u64(payload, 56),
            ))
        }
        SECURITY_BINDING_PRESENT => {
            let policy_identity = payload[72..104].try_into().unwrap();
            let retention = read_u64(payload, 104);
            if policy_identity == [0; 32] {
                return Err(CheckpointStreamDecodeDenial::InvalidSecurityPolicyIdentity);
            }
            if retention == 0 {
                return Err(CheckpointStreamDecodeDenial::InvalidSecurityRetention);
            }
            let source = PhysicalCheckpointSource::secured_concurrent(
                identity,
                wal,
                root,
                read_u64(payload, 56),
                policy_identity,
                retention,
            )
            .expect("nonzero decoded security fields satisfy the binding constructor");
            if source.security_binding().unwrap().digest() != payload[112..144] {
                return Err(CheckpointStreamDecodeDenial::SecurityBindingDigestMismatch);
            }
            Ok(source)
        }
        observed => Err(CheckpointStreamDecodeDenial::InvalidSecurityBindingPresence(observed)),
    }
}

fn security_digest(
    identity: PhysicalCheckpointIdentity,
    wal: CheckpointWalSourceRange,
    root: CheckpointRootBasis,
    policy_identity: [u8; 32],
    retention: u64,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.checkpoint-security-binding.v1");
    digest.update(identity.store_identity().bytes());
    digest.update(identity.sequence().get().to_le_bytes());
    digest.update(wal.admitted_begin_lsn().to_le_bytes());
    digest.update(wal.covered_end_lsn_exclusive().to_le_bytes());
    digest.update(root.generation().to_le_bytes());
    digest.update(root.tree_identity().to_le_bytes());
    digest.update(policy_identity);
    digest.update(retention.to_le_bytes());
    digest.finalize().into()
}
