//! Schema-affine field locus for retained inverse material.
use super::PortablePreImageLocus;
use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, DeclaredApplicationFieldValue,
};
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
/// A schema-affine application-field identity required by a recorded inverse.
///
/// The schema witness is retained until the owning operation-definition
/// builder associates the complete aftermath contract with that same schema.
pub struct DeclaredPreImageLocus<Schema> {
    portable: PortablePreImageLocus,
    schema: PhantomData<fn() -> Schema>,
}

impl<Schema> DeclaredPreImageLocus<Schema> {
    pub fn from_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        Self {
            portable: PortablePreImageLocus {
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            },
            schema: PhantomData,
        }
    }

    pub fn entity(&self) -> &str {
        self.portable.entity()
    }

    pub fn aspect(&self) -> &str {
        self.portable.aspect()
    }

    pub fn field(&self) -> &str {
        self.portable.field()
    }

    pub(super) fn into_portable(self) -> PortablePreImageLocus {
        self.portable
    }
}

impl<Schema> Clone for DeclaredPreImageLocus<Schema> {
    fn clone(&self) -> Self {
        Self {
            portable: self.portable.clone(),
            schema: PhantomData,
        }
    }
}

impl<Schema> fmt::Debug for DeclaredPreImageLocus<Schema> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.portable.fmt(formatter)
    }
}

impl<Schema> PartialEq for DeclaredPreImageLocus<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.portable == other.portable
    }
}

impl<Schema> Eq for DeclaredPreImageLocus<Schema> {}

impl<Schema> PartialOrd for DeclaredPreImageLocus<Schema> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Schema> Ord for DeclaredPreImageLocus<Schema> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.portable.cmp(&other.portable)
    }
}

impl<Schema> Hash for DeclaredPreImageLocus<Schema> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.portable.hash(state);
    }
}
