//! Idempotency commitment; live epoch issuance remains runtime-affine.
use super::{WorthQueryObservedSourceCoordinate, WorthQueryObservedSourceOccurrence};
use sha2::{Digest, Sha256};

/// Product Home commitments survive readmission; unqualified Native and
/// MemoryOnly observations remain scoped to their live Query authority.
pub(super) fn stable_runtime_identity(
    runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    coordinate: &WorthQueryObservedSourceCoordinate,
    checkpoint_identity: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:runtime-source-commitment:v2");
    match coordinate.occurrence {
        WorthQueryObservedSourceOccurrence::Relational => {
            digest.update([0]);
            digest.update(runtime_authority.as_u64().to_be_bytes());
        }
        WorthQueryObservedSourceOccurrence::Product(occurrence) => {
            match occurrence.owner_identity().durable() {
                Some(owner) => {
                    return home_source_identity(
                        digest,
                        owner,
                        occurrence.ordinal(),
                        checkpoint_identity,
                    )
                }
                None => {
                    digest.update([1]);
                    digest.update(runtime_authority.as_u64().to_be_bytes());
                    digest.update(occurrence.owner_identity().get().to_be_bytes());
                    digest.update(occurrence.ordinal().to_be_bytes());
                }
            }
        }
    }
    digest.update(checkpoint_identity);
    digest.finalize().into()
}

fn home_source_identity(
    mut digest: Sha256,
    owner: worth_foundational::facade::DurableWorldOwnerIdentity,
    incarnation: u64,
    checkpoint: &[u8; 32],
) -> [u8; 32] {
    digest.update([2]);
    digest.update(owner.home().bytes());
    digest.update(owner.slot().get().to_be_bytes());
    digest.update(incarnation.to_be_bytes());
    digest.update(checkpoint);
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{
        DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity,
    };
    #[test]
    fn home_commitment_preserves_home_owner_incarnation_and_semantic_axes() {
        // These are descriptive hash inputs, never live World admission.
        let compute = |home, slot, incarnation, checkpoint| {
            let owner = DurableWorldOwnerIdentity::from_recorded(
                HomeIdentity::from_observed_bytes([home; 16]).unwrap(),
                WorldOwnerSlotIdentity::from_recorded(slot),
            );
            let mut digest = Sha256::new();
            digest.update(b"worth-query:runtime-source-commitment:v2");
            home_source_identity(digest, owner, incarnation, &[checkpoint; 32])
        };
        let original = compute(1, 7, 11, 3);
        // Independent fixed-width framing of the portable meaning.
        assert_eq!(
            original,
            [
                32, 117, 185, 217, 156, 99, 243, 46, 4, 178, 164, 167, 61, 130, 179, 167, 213, 186,
                219, 172, 172, 172, 31, 170, 191, 193, 4, 143, 8, 88, 85, 44
            ]
        );
        assert_ne!(original, compute(2, 7, 11, 3));
        assert_ne!(original, compute(1, 8, 11, 3));
        assert_ne!(original, compute(1, 7, 12, 3));
        assert_ne!(original, compute(1, 7, 11, 4));
    }
}
