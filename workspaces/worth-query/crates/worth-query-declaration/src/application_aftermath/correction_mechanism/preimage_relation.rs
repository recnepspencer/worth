//! Schema-affine relation kind and endpoint roles for historical inverse material.
use crate::application_schema::ApplicationRelationRef;
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PortablePreImageRelationLocus {
    relation: String,
    from: String,
    to: String,
}
impl PortablePreImageRelationLocus {
    pub fn from_untrusted_fields(relation: String, from: String, to: String) -> Self {
        Self { relation, from, to }
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn from(&self) -> &str {
        &self.from
    }
    pub fn to(&self) -> &str {
        &self.to
    }
}
/// Historical endpoints grant no current identity or mutation authority.
pub struct DeclaredPreImageRelationLocus<Schema> {
    portable: PortablePreImageRelationLocus,
    schema: PhantomData<fn() -> Schema>,
}
impl<Schema> DeclaredPreImageRelationLocus<Schema> {
    pub fn from_relation<Relation, From, To>(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self {
            portable: PortablePreImageRelationLocus::from_untrusted_fields(
                relation.name().to_owned(),
                relation.from().to_owned(),
                relation.to().to_owned(),
            ),
            schema: PhantomData,
        }
    }
    pub fn relation(&self) -> &str {
        self.portable.relation()
    }
    pub fn from(&self) -> &str {
        self.portable.from()
    }
    pub fn to(&self) -> &str {
        self.portable.to()
    }
    pub(super) fn into_portable(self) -> PortablePreImageRelationLocus {
        self.portable
    }
}
impl<S> Clone for DeclaredPreImageRelationLocus<S> {
    fn clone(&self) -> Self {
        Self {
            portable: self.portable.clone(),
            schema: PhantomData,
        }
    }
}
impl<S> fmt::Debug for DeclaredPreImageRelationLocus<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.portable.fmt(f)
    }
}
impl<S> PartialEq for DeclaredPreImageRelationLocus<S> {
    fn eq(&self, other: &Self) -> bool {
        self.portable == other.portable
    }
}
impl<S> Eq for DeclaredPreImageRelationLocus<S> {}
impl<S> PartialOrd for DeclaredPreImageRelationLocus<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<S> Ord for DeclaredPreImageRelationLocus<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.portable.cmp(&other.portable)
    }
}
impl<S> Hash for DeclaredPreImageRelationLocus<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.portable.hash(state);
    }
}
