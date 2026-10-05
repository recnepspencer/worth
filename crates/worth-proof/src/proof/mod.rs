mod execution_authority;
mod marker_authoring;
mod markers;
mod minting;
mod sets;
mod structural_facts;
mod witnesses;

pub use execution_authority::ExecutionAuthorityMarker;
pub use markers::ProofMarker;
#[cfg(test)]
pub(crate) use minting::mint_proof;
pub use sets::{AuthorityProves, NoProofs, Proof, ProofSet, ProofSetAuthorizedBy, ProofSetCons};
pub use structural_facts::{
    CanonicalOrder, Disjointness, Normalization, StructuralProofAuthority, Uniqueness,
};
#[cfg(test)]
pub(crate) use witnesses::{mint_authority_witness, mint_capability_witness};
pub use witnesses::{AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness};
