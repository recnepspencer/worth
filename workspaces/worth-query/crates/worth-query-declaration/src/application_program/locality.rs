use std::any::TypeId;

/// Domain-neutral shape of the region within which an operation or artifact is meaningful.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationLocalityGranule {
    Root,
    Partition,
    Neighborhood,
}

/// A domain-owned locality meaning attached to canonical application-program meaning.
///
/// Implementations describe scope only. They grant no traversal, read, mutation, or
/// publication authority.
pub trait ApplicationLocalityScope: Sized + 'static {
    const IDENTITY: &'static str;
    const GRANULE: ApplicationLocalityGranule;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationLocalityDeclaration {
    identity: &'static str,
    granule: ApplicationLocalityGranule,
    scope_type: TypeId,
}

impl ApplicationLocalityDeclaration {
    pub(crate) fn of<Scope: ApplicationLocalityScope>() -> Self {
        Self {
            identity: Scope::IDENTITY,
            granule: Scope::GRANULE,
            scope_type: TypeId::of::<Scope>(),
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }

    pub const fn granule(&self) -> ApplicationLocalityGranule {
        self.granule
    }

    pub const fn scope_type(&self) -> TypeId {
        self.scope_type
    }
}
