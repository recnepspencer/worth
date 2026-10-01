//! A purpose-specific WAL statement for switching the arena namespace to
//! deterministic tier classes. An intent alone never activates the epoch;
//! only a checksum-bound selected candidate root does so.

use sha2::{Digest, Sha256};

pub const TIER_EPOCH_ACTIVATION_DOMAIN: &[u8] = b"store.physical.tier-epoch-activation.v1";
const BODY_BYTES: usize = 168;
pub const TIER_EPOCH_ACTIVATION_WIRE_BYTES: usize =
    8 + TIER_EPOCH_ACTIVATION_DOMAIN.len() + 1 + BODY_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierEpochActivationPhaseV1 {
    Intent,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TierEpochActivationDenial {
    Malformed,
    InvalidBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierEpochActivationV1 {
    phase: TierEpochActivationPhaseV1,
    store: [u8; 16],
    attempt: [u8; 16],
    source_root_generation: u64,
    source_root_sha256: [u8; 32],
    source_free_sha256: [u8; 32],
    tier_epoch_start: u64,
    candidate_root_generation: u64,
    candidate_root_sha256: [u8; 32],
    retained_metadata_bytes: u64,
    publication: u64,
}

impl TierEpochActivationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn intent(
        store: [u8; 16],
        attempt: [u8; 16],
        source_root_generation: u64,
        source_root_sha256: [u8; 32],
        source_free_sha256: [u8; 32],
        tier_epoch_start: u64,
        candidate_root_generation: u64,
        candidate_root_sha256: [u8; 32],
        retained_metadata_bytes: u64,
        publication: u64,
    ) -> Result<Self, TierEpochActivationDenial> {
        if store == [0; 16]
            || attempt == [0; 16]
            || source_root_generation == 0
            || source_root_sha256 == [0; 32]
            || source_free_sha256 == [0; 32]
            || tier_epoch_start == 0
            || source_root_generation.checked_add(1) != Some(candidate_root_generation)
            || candidate_root_sha256 == [0; 32]
            || retained_metadata_bytes == 0
            || publication == 0
        {
            return Err(TierEpochActivationDenial::InvalidBinding);
        }
        Ok(Self {
            phase: TierEpochActivationPhaseV1::Intent,
            store,
            attempt,
            source_root_generation,
            source_root_sha256,
            source_free_sha256,
            tier_epoch_start,
            candidate_root_generation,
            candidate_root_sha256,
            retained_metadata_bytes,
            publication,
        })
    }

    pub const fn completed(mut self) -> Self {
        self.phase = TierEpochActivationPhaseV1::Completed;
        self
    }

    pub fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(TIER_EPOCH_ACTIVATION_WIRE_BYTES);
        bytes.extend_from_slice(&(TIER_EPOCH_ACTIVATION_DOMAIN.len() as u64).to_le_bytes());
        bytes.extend_from_slice(TIER_EPOCH_ACTIVATION_DOMAIN);
        bytes.push(match self.phase {
            TierEpochActivationPhaseV1::Intent => 1,
            TierEpochActivationPhaseV1::Completed => 2,
        });
        bytes.extend_from_slice(&self.store);
        bytes.extend_from_slice(&self.attempt);
        bytes.extend_from_slice(&self.source_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.source_root_sha256);
        bytes.extend_from_slice(&self.source_free_sha256);
        bytes.extend_from_slice(&self.tier_epoch_start.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_generation.to_le_bytes());
        bytes.extend_from_slice(&self.candidate_root_sha256);
        bytes.extend_from_slice(&self.retained_metadata_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.publication.to_le_bytes());
        debug_assert_eq!(bytes.len(), TIER_EPOCH_ACTIVATION_WIRE_BYTES);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, TierEpochActivationDenial> {
        if bytes.len() != TIER_EPOCH_ACTIVATION_WIRE_BYTES
            || !payload_is_tier_epoch_activation(bytes)
        {
            return Err(TierEpochActivationDenial::Malformed);
        }
        let mut cursor = 8 + TIER_EPOCH_ACTIVATION_DOMAIN.len();
        let phase = match take::<1>(bytes, &mut cursor)[0] {
            1 => TierEpochActivationPhaseV1::Intent,
            2 => TierEpochActivationPhaseV1::Completed,
            _ => return Err(TierEpochActivationDenial::Malformed),
        };
        let store = take::<16>(bytes, &mut cursor);
        let attempt = take::<16>(bytes, &mut cursor);
        let source_generation = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let source_root_sha256 = take::<32>(bytes, &mut cursor);
        let source_free_sha256 = take::<32>(bytes, &mut cursor);
        let epoch = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate_generation = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let candidate_root_sha256 = take::<32>(bytes, &mut cursor);
        let retained_metadata_bytes = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let publication = u64::from_le_bytes(take::<8>(bytes, &mut cursor));
        let mut intent = Self::intent(
            store,
            attempt,
            source_generation,
            source_root_sha256,
            source_free_sha256,
            epoch,
            candidate_generation,
            candidate_root_sha256,
            retained_metadata_bytes,
            publication,
        )?;
        intent.phase = phase;
        Ok(intent)
    }

    pub const fn phase(self) -> TierEpochActivationPhaseV1 {
        self.phase
    }
    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn attempt(self) -> [u8; 16] {
        self.attempt
    }
    pub const fn source_root_generation(self) -> u64 {
        self.source_root_generation
    }
    pub const fn source_root_sha256(self) -> [u8; 32] {
        self.source_root_sha256
    }
    pub const fn source_free_sha256(self) -> [u8; 32] {
        self.source_free_sha256
    }
    pub const fn tier_epoch_start(self) -> u64 {
        self.tier_epoch_start
    }
    pub const fn candidate_root_generation(self) -> u64 {
        self.candidate_root_generation
    }
    pub const fn candidate_root_sha256(self) -> [u8; 32] {
        self.candidate_root_sha256
    }
    pub const fn retained_metadata_bytes(self) -> u64 {
        self.retained_metadata_bytes
    }
    pub const fn publication(self) -> u64 {
        self.publication
    }

    /// Stable root-carried identity, deliberately independent of the candidate
    /// digest so the candidate can include it without a hash cycle.
    pub fn epoch_anchor(self) -> [u8; 32] {
        tier_epoch_anchor(
            self.store,
            self.attempt,
            self.source_root_generation,
            self.source_root_sha256,
            self.source_free_sha256,
            self.tier_epoch_start,
        )
        .expect("validated activation has an anchor")
    }
}

pub fn tier_epoch_anchor(
    store: [u8; 16],
    attempt: [u8; 16],
    source_root_generation: u64,
    source_root_sha256: [u8; 32],
    source_free_sha256: [u8; 32],
    epoch: u64,
) -> Option<[u8; 32]> {
    if store == [0; 16]
        || attempt == [0; 16]
        || source_root_generation == 0
        || source_root_sha256 == [0; 32]
        || source_free_sha256 == [0; 32]
        || epoch == 0
        || source_root_generation == u64::MAX
    {
        return None;
    }
    let mut digest = Sha256::new();
    digest.update(b"worth.store.tier-epoch-root-anchor.v1");
    digest.update(store);
    digest.update(attempt);
    digest.update(source_root_generation.to_le_bytes());
    digest.update(source_root_sha256);
    digest.update(source_free_sha256);
    digest.update(epoch.to_le_bytes());
    Some(digest.finalize().into())
}

pub fn payload_is_tier_epoch_activation(bytes: &[u8]) -> bool {
    bytes.len() >= 8 + TIER_EPOCH_ACTIVATION_DOMAIN.len()
        && bytes[..8] == (TIER_EPOCH_ACTIVATION_DOMAIN.len() as u64).to_le_bytes()
        && &bytes[8..8 + TIER_EPOCH_ACTIVATION_DOMAIN.len()] == TIER_EPOCH_ACTIVATION_DOMAIN
}

fn take<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let result = bytes[*cursor..*cursor + N].try_into().unwrap();
    *cursor += N;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_intent_and_completion_roundtrip_without_receipt_substitution() {
        let intent = TierEpochActivationV1::intent(
            [1; 16], [2; 16], 3, [4; 32], [5; 32], 7, 4, [6; 32], 4096, 8,
        )
        .unwrap();
        assert_eq!(TierEpochActivationV1::decode(&intent.encode()), Ok(intent));
        assert_eq!(
            TierEpochActivationV1::decode(&intent.completed().encode()),
            Ok(intent.completed())
        );
        assert_eq!(intent.epoch_anchor(), intent.completed().epoch_anchor());
        assert_ne!(
            intent.epoch_anchor(),
            tier_epoch_anchor([1; 16], [2; 16], 3, [4; 32], [5; 32], 8,).unwrap()
        );
        assert!(!payload_is_tier_epoch_activation(
            b"store.physical.blob-manifest-residue-cleanup.v1"
        ));
        assert!(TierEpochActivationV1::intent(
            [1; 16], [2; 16], 3, [4; 32], [5; 32], 0, 4, [6; 32], 4096, 8,
        )
        .is_err());
    }
}
