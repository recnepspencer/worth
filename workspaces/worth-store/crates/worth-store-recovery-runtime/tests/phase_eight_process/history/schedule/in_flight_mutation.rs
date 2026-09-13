use sha2::{Digest, Sha256};

const CHECKPOINT_PAGE_BYTES: usize = 16 * 1024;
const EXTENT_OVERHEAD_BYTES: usize = 112;
const ORDINARY_PAYLOAD_BYTES: usize = 8 * 1024;
const INLINE_RECORD_PAYLOAD_BYTES: usize = 128;
const CHECKPOINT_PAYLOAD_BYTES: usize = 2 * (CHECKPOINT_PAGE_BYTES - EXTENT_OVERHEAD_BYTES) + 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedInFlightMutation {
    perturbation_seed: u64,
    material: [u8; 32],
    payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MutationCrashWorkload {
    ExtentWriteback,
    InlineRecord,
    CapacityTransition,
}

impl MutationCrashWorkload {
    pub(crate) const fn cli_name(self) -> &'static str {
        match self {
            Self::ExtentWriteback => "extent-writeback",
            Self::InlineRecord => "inline-record",
            Self::CapacityTransition => "capacity-transition",
        }
    }

    const fn payload_bytes(self) -> usize {
        match self {
            Self::ExtentWriteback => CHECKPOINT_PAYLOAD_BYTES,
            Self::InlineRecord => INLINE_RECORD_PAYLOAD_BYTES,
            Self::CapacityTransition => INLINE_RECORD_PAYLOAD_BYTES,
        }
    }
}

impl ExpectedInFlightMutation {
    pub(crate) fn checkpoint_exact_prefix(seed: u64) -> Self {
        Self::new(seed, CHECKPOINT_PAYLOAD_BYTES)
    }

    pub(crate) fn durable_before_ack(seed: u64) -> Self {
        Self::new(seed, ORDINARY_PAYLOAD_BYTES)
    }

    pub(crate) fn mutation_crash(seed: u64, workload: MutationCrashWorkload) -> Self {
        Self::new(seed, workload.payload_bytes())
    }

    pub(crate) const fn material(&self) -> [u8; 32] {
        self.material
    }

    pub(crate) const fn perturbation_seed(&self) -> u64 {
        self.perturbation_seed
    }

    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn new(seed: u64, payload_bytes: usize) -> Self {
        let material = dirty_material(seed);
        Self {
            perturbation_seed: seed,
            material,
            payload: payload(material, payload_bytes),
        }
    }
}

fn dirty_material(seed: u64) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.c8.dirty-mutation.v1");
    digest.update(seed.to_le_bytes());
    digest.finalize().into()
}

fn payload(material: [u8; 32], length: usize) -> Vec<u8> {
    let mut payload = vec![0; length];
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte = material[index % material.len()]
            .wrapping_add(index as u8)
            .wrapping_mul(31);
    }
    payload
}
