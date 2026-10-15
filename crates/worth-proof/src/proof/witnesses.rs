use std::marker::PhantomData;

pub trait AuthorityMarker: 'static {}

pub trait CapabilityMarker: 'static {}

pub struct AuthorityWitness<A>(PhantomData<A>)
where
    A: AuthorityMarker;

// Not derived, so an authority marker never needs `PartialEq` to let two
// witnesses be compared. Deliberately **not** `Copy`: a witness is passed where
// authority is exercised, and silent duplication is not a property this type
// should hand out for free.
crate::type_level_traits!(AuthorityWitness<A: AuthorityMarker>);

impl<A> AuthorityWitness<A>
where
    A: AuthorityMarker,
{
    #[cfg(test)]
    pub(crate) fn mint() -> Self {
        Self(PhantomData)
    }

    /// Mint a witness by surrendering a value of the marker type.
    ///
    /// **The witness is exactly as sealed as your marker's constructor.** This
    /// crate cannot seal it for you: `AuthorityMarker` is an open trait, and
    /// this function accepts any `A` implementing it. Sealing is delegated, by
    /// design — possession of an `A` *is* the authority.
    ///
    /// That makes the marker's visibility load-bearing:
    ///
    /// ```text
    /// pub struct MyAuthority;      // NOT a seal — any consumer can construct it
    /// pub struct MyAuthority(());  // a seal — only the owning module can
    /// struct MyAuthority;          // a seal — private to its module
    /// ```
    ///
    /// Prefer [`crate::authority_marker!`], which generates the sealed shape so
    /// the correct pattern is the default rather than something each consumer
    /// has to know. See `tests/ui/milestone2/` for the compile-fail and
    /// compile-pass cases that hold this contract to its word.
    pub fn from_authority_marker(_marker: A) -> Self {
        Self(PhantomData)
    }
}

pub struct CapabilityWitness<C>(PhantomData<C>)
where
    C: CapabilityMarker;

crate::type_level_traits!(CapabilityWitness<C: CapabilityMarker>);

impl<C> CapabilityWitness<C>
where
    C: CapabilityMarker,
{
    #[cfg(test)]
    pub(crate) fn mint() -> Self {
        Self(PhantomData)
    }

    /// Mint a witness by surrendering a value of the marker type.
    ///
    /// **The witness is exactly as sealed as your marker's constructor** — see
    /// [`AuthorityWitness::from_authority_marker`] for the full contract and
    /// [`crate::capability_marker!`] for the sealed shape.
    pub fn from_capability_marker(_marker: C) -> Self {
        Self(PhantomData)
    }
}

#[cfg(test)]
pub(crate) fn mint_authority_witness<A>() -> AuthorityWitness<A>
where
    A: AuthorityMarker,
{
    AuthorityWitness::mint()
}

#[cfg(test)]
pub(crate) fn mint_capability_witness<C>() -> CapabilityWitness<C>
where
    C: CapabilityMarker,
{
    CapabilityWitness::mint()
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness};

    struct DeploymentAuthority;
    impl AuthorityMarker for DeploymentAuthority {}

    struct CanonicalizationCapability;
    impl CapabilityMarker for CanonicalizationCapability {}

    const _: () = assert!(size_of::<AuthorityWitness<DeploymentAuthority>>() == 0);
    const _: () = assert!(size_of::<CapabilityWitness<CanonicalizationCapability>>() == 0);
}
