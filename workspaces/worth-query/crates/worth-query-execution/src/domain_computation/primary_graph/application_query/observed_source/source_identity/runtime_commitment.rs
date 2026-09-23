use sha2::{Digest, Sha256};

use super::{WorthQueryObservedSourceCoordinate, WorthQueryObservedSourceOccurrence};

/// Stable within one runtime and branch occurrence for equivalent native meaning.
/// The meaning object's issuance identity remains separate for epoch custody.
pub(super) fn stable_runtime_identity(
    runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    coordinate: &WorthQueryObservedSourceCoordinate,
    checkpoint_identity: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:runtime-source-commitment:v1");
    digest.update(runtime_authority.as_u64().to_be_bytes());
    match coordinate.occurrence {
        WorthQueryObservedSourceOccurrence::Relational => digest.update([0]),
        WorthQueryObservedSourceOccurrence::Product(occurrence) => {
            digest.update([1]);
            digest.update(occurrence.owner_identity().get().to_be_bytes());
            digest.update(occurrence.ordinal().to_be_bytes());
        }
    }
    digest.update(checkpoint_identity);
    digest.finalize().into()
}
