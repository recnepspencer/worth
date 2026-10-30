//! The live witness door, held to its contract.
//!
//! `witnesses_are_not_publicly_mintable.rs` covers `AuthorityWitness::mint`,
//! which is crate-internal. This covers `from_authority_marker`, which is `pub`
//! and is the door consumers actually reach. A witness is exactly as sealed as
//! its marker's constructor, so a marker produced by `authority_marker!` must
//! be unmintable from outside the module that declared it.
//!
//! The probe is a **sibling** module. Rust privacy reaches the declaring module
//! *and its descendants*, so a child module of the declarer legitimately sees
//! the seal â€” that is the contract, not a hole. A sibling is the nearest
//! position that must be refused.

mod owner {
    worth_proof::authority_marker!(pub SealedAuthority);
    worth_proof::capability_marker!(pub SealedCapability);
}

mod outsider {
    use worth_proof::{AuthorityWitness, CapabilityWitness};

    pub fn attempt_struct_literal() -> AuthorityWitness<super::owner::SealedAuthority> {
        let authority = super::owner::SealedAuthority(::core::marker::PhantomData);
        AuthorityWitness::from_authority_marker(authority)
    }

    pub fn attempt_capability_struct_literal() -> CapabilityWitness<super::owner::SealedCapability>
    {
        let capability = super::owner::SealedCapability(::core::marker::PhantomData);
        CapabilityWitness::from_capability_marker(capability)
    }

    pub fn attempt_private_seal() -> AuthorityWitness<super::owner::SealedAuthority> {
        AuthorityWitness::from_authority_marker(super::owner::SealedAuthority::seal())
    }

    pub fn attempt_private_witness() -> AuthorityWitness<super::owner::SealedAuthority> {
        super::owner::SealedAuthority::witness()
    }
}

fn main() {
    let _ = outsider::attempt_struct_literal();
    let _ = outsider::attempt_capability_struct_literal();
    let _ = outsider::attempt_private_seal();
    let _ = outsider::attempt_private_witness();
}
// sealed-minting-case
