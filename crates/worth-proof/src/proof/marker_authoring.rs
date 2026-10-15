//! Sealed marker authoring.
//!
//! [`AuthorityWitness::from_authority_marker`] is exactly as sealed as its
//! marker's constructor, because possession of the marker *is* the authority.
//! That contract is easy to state and easy to get wrong: a `pub struct Foo;` is
//! freely constructible by any consumer, so a witness over it proves nothing.
//!
//! These macros generate the sealed shape, so the correct pattern is the
//! default rather than something each consumer has to know.
//!
//! [`AuthorityWitness::from_authority_marker`]: crate::AuthorityWitness::from_authority_marker

/// Declare an authority marker that only its defining module can construct.
///
/// The generated type may be `pub` — it has to be nameable, since consumers
/// write `AuthorityWitness<MyAuthority>` in signatures — while its single field
/// stays private. `seal()` and `witness()` carry no visibility modifier for the
/// same reason: minting is the owner's privilege, and delegating it is a
/// decision the owner makes explicitly by writing a wrapper, never one this
/// macro makes for them.
///
/// **The exact reach of the seal:** Rust privacy admits the declaring module
/// *and its descendants*. A child module of the declarer can mint; a sibling,
/// a parent, and any other crate cannot. Declare the marker in the module that
/// owns the authority, not in a parent that happens to be convenient — nesting
/// a submodule under it grants that submodule minting rights. `tests/ui/
/// milestone2/sealed_markers_are_not_mintable_by_consumers.rs` pins the sibling
/// boundary.
///
/// ```
/// worth_proof::authority_marker!(pub ResolutionAuthority);
///
/// mod owner {
///     worth_proof::authority_marker!(pub DeploymentAuthority);
///
///     // Only this module can mint, because only this module can construct.
///     pub fn authorize() -> worth_proof::AuthorityWitness<DeploymentAuthority> {
///         DeploymentAuthority::witness()
///     }
/// }
///
/// let _witness = owner::authorize();
/// ```
#[macro_export]
macro_rules! authority_marker {
    ($(#[$meta:meta])* $vis:vis $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, PartialEq, Eq)]
        $vis struct $name(::core::marker::PhantomData<()>);

        impl $crate::AuthorityMarker for $name {}

        impl $name {
            /// Construct the marker. Private to the declaring module — the
            /// private field is the seal.
            #[allow(dead_code)]
            fn seal() -> Self {
                Self(::core::marker::PhantomData)
            }

            /// Mint a witness for this authority. Private to the declaring
            /// module; expose a wrapper deliberately if callers should have it.
            #[allow(dead_code)]
            fn witness() -> $crate::AuthorityWitness<Self> {
                $crate::AuthorityWitness::from_authority_marker(Self::seal())
            }
        }
    };
}

/// Declare a capability marker that only its defining module can construct.
///
/// The capability counterpart of [`authority_marker!`]; the same sealing rules
/// apply.
///
/// ```
/// worth_proof::capability_marker!(pub CanonicalizationCapability);
/// ```
#[macro_export]
macro_rules! capability_marker {
    ($(#[$meta:meta])* $vis:vis $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, PartialEq, Eq)]
        $vis struct $name(::core::marker::PhantomData<()>);

        impl $crate::CapabilityMarker for $name {}

        impl $name {
            /// Construct the marker. Private to the declaring module — the
            /// private field is the seal.
            #[allow(dead_code)]
            fn seal() -> Self {
                Self(::core::marker::PhantomData)
            }

            /// Mint a witness for this capability. Private to the declaring
            /// module; expose a wrapper deliberately if callers should have it.
            #[allow(dead_code)]
            fn witness() -> $crate::CapabilityWitness<Self> {
                $crate::CapabilityWitness::from_capability_marker(Self::seal())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    crate::authority_marker!(pub SealedAuthority);
    crate::capability_marker!(pub SealedCapability);

    // The seal must cost nothing: a marker carrying a private PhantomData
    // is still a ZST, so sealing does not trade away the crate's
    // static-first guarantee.
    const _: () = assert!(size_of::<SealedAuthority>() == 0);
    const _: () = assert!(size_of::<SealedCapability>() == 0);
    const _: () = assert!(size_of::<crate::AuthorityWitness<SealedAuthority>>() == 0);
    const _: () = assert!(size_of::<crate::CapabilityWitness<SealedCapability>>() == 0);

    #[test]
    fn declaring_module_can_mint_through_the_seal() {
        let _authority = SealedAuthority::witness();
        let _capability = SealedCapability::witness();
    }
}
