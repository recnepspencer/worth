/// Rust type identity permits equality and point lookup, never ordering.
/// The snippets include the private production type inside their own crate;
/// refusal therefore tests ordering, not access through a public facade.
///
/// ```compile_fail,E0369
/// mod identity {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/equality_type_id.rs"));
/// }
/// let a = identity::EqualityTypeId::of::<u8>();
/// let b = identity::EqualityTypeId::of::<u16>();
/// let _ = a < b;
/// ```
///
/// ```compile_fail,E0277
/// mod identity {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/equality_type_id.rs"));
/// }
/// let mut keys = [identity::EqualityTypeId::of::<u8>(), identity::EqualityTypeId::of::<u16>()];
/// keys.sort();
/// ```
///
/// ```
/// mod identity {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/equality_type_id.rs"));
/// }
/// let a = identity::EqualityTypeId::of::<u8>();
/// let b = identity::EqualityTypeId::of::<u16>();
/// assert_ne!(a, b);
/// let mut members = std::collections::HashSet::new();
/// members.insert(a);
/// assert!(members.contains(&identity::EqualityTypeId::of::<u8>()));
/// ```
#[derive(PartialEq, Eq, Hash)]
pub(crate) struct EqualityTypeId(std::any::TypeId);

impl EqualityTypeId {
    pub(crate) fn of<T: ?Sized + 'static>() -> Self {
        Self(std::any::TypeId::of::<T>())
    }

    /// Converts only at an existing external TypeId contract.
    pub(crate) const fn into_external(self) -> std::any::TypeId {
        self.0
    }
}

impl From<std::any::TypeId> for EqualityTypeId {
    fn from(identity: std::any::TypeId) -> Self {
        Self(identity)
    }
}

impl Copy for EqualityTypeId {}

impl Clone for EqualityTypeId {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::fmt::Debug for EqualityTypeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
