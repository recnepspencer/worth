//! Pure correlation; only the installed owner can attest this selected source.
use crate::{
    branch::ProductBranchObservation,
    durability::{WorldDurableMediumDenial, WorldHeadInput},
};
use sha2::{Digest, Sha256};
pub fn world_branch_program_source_commitment(
    expected: &ProductBranchObservation,
    revision: [u8; 32],
) -> Result<[u8; 32], WorldDurableMediumDenial> {
    let head = WorldHeadInput::from_observation(expected)?;
    let mut hash = Sha256::new();
    hash.update(b"worth.world.branch-program-source.v1");
    hash.update(head.owner.home().bytes());
    hash.update(head.owner.slot().get().to_le_bytes());
    hash.update((head.name.len() as u64).to_le_bytes());
    hash.update(head.name.as_bytes());
    hash.update(head.incarnation.ordinal().to_le_bytes());
    hash.update(head.generation.get().to_le_bytes());
    hash.update(head.commit.ordinal().to_le_bytes());
    hash.update(revision);
    Ok(hash.finalize().into())
}
