//! Authoring selects a record field; schema compilation checks the selection.
use super::{
    Aspect, Entity, Fragment, FragmentCatalog, In, Record, RecordField, RecordFieldDefinition,
    Scalar,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationSchemaDeclarationDenial,
};
use std::{any::TypeId, marker::PhantomData};

type Resolve<E, A, T> =
    fn(&str) -> Result<RecordField<E, A, T>, ApplicationSchemaDeclarationDenial>;

/// A declaration expression, carrying no installed authority.
/// Its owning fragment supplies the binding, rather than a second field marker.
pub struct FieldSelection<F, E, A, T> {
    path: &'static str,
    resolve: Resolve<E, A, T>,
    fragment: PhantomData<fn() -> F>,
}
impl<F, E, A, T> Copy for FieldSelection<F, E, A, T> {}
impl<F, E, A, T> Clone for FieldSelection<F, E, A, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<F, E, A, T> FieldSelection<F, E, A, T> {
    /// Checks the path against the record during declaration construction.
    pub fn resolve(self) -> Result<RecordField<E, A, T>, ApplicationSchemaDeclarationDenial> {
        (self.resolve)(self.path)
    }
    pub const fn path(self) -> &'static str {
        self.path
    }
}
impl<F: Fragment, E: Entity, A: Aspect, T: 'static> FieldSelection<F, E, A, T>
where
    F::Bindings: Scalar<T>,
{
    pub fn entity(self) -> &'static str {
        E::NAME
    }
    pub fn aspect(self) -> &'static str {
        super::native_aspect_key::<E, A>()
    }
    pub const fn field(self) -> &'static str {
        self.path
    }
    pub fn value_type_name(self) -> &'static str {
        <<F::Bindings as Scalar<T>>::Binding as crate::application_schema::ApplicationScalarValueBinding>::IDENTITY_NAME
    }
    pub fn unit(self) -> Option<&'static str> {
        <<F::Bindings as Scalar<T>>::Binding as crate::application_schema::ApplicationScalarValueBinding>::UNIT.map(|unit| unit.as_str())
    }
    pub fn scalar_family(self) -> worth_foundational::facade::ScalarAspectType {
        <<F::Bindings as Scalar<T>>::Binding as crate::application_schema::ApplicationScalarValueBinding>::SCALAR_FAMILY
    }
    /// Selects a named column from this record, proving both memberships.
    pub const fn at<EP, AP>(path: &'static str) -> Self
    where
        E: In<F::Members, EP, F>,
        A: In<E::Aspects, AP, E>,
        F::Bindings: Record<A::Row>,
    {
        Self {
            path,
            resolve: super::field_handle::checked_record_field::<F, E, A, T>,
            fragment: PhantomData,
        }
    }
    /// The selection's expected codec; installation checks it against the column.
    pub fn binding_recipe(self) -> ApplicationFieldBindingRecipe {
        ApplicationFieldBindingRecipe::of::<<F::Bindings as Scalar<T>>::Binding>(
            ApplicationFieldBindingLocus::new(
                E::NAME,
                super::native_aspect_key::<E, A>(),
                self.path,
            ),
        )
    }
    /// Ordinary access reuses installed metadata without reconstructing a shape.
    pub fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&RecordFieldDefinition, ApplicationSchemaDeclarationDenial> {
        catalog
            .fragment_type::<F>()
            .and_then(|fragment| fragment.aspect::<E, A>())
            .and_then(|aspect| {
                aspect.fields().iter().find(|field| {
                    !field.internal_presence()
                        && field.recipe().locus().field() == self.path
                        && field.recipe().value_type() == TypeId::of::<T>()
                        && field.recipe().binding_type()
                            == TypeId::of::<<F::Bindings as Scalar<T>>::Binding>()
                })
            })
            .or_else(|| {
                // A name-derived field identity survives a Rust-only marker or
                // policy-type change. A View still requires its actual fragment.
                if catalog.fragment_type::<F>().is_some() {
                    return None;
                }
                catalog.named_field::<F, E, A, T>(
                    self.path,
                    TypeId::of::<<F::Bindings as Scalar<T>>::Binding>(),
                )
            })
            .ok_or_else(|| ApplicationSchemaDeclarationDenial::InvalidRecordField {
                field: self.binding_recipe().locus().clone(),
            })
    }
}

pub fn select_field<F, E, A, T, EP, AP>(
    _: F,
    _: E,
    _: A,
    path: &'static str,
) -> FieldSelection<F, E, A, T>
where
    F: Fragment,
    E: Entity + In<F::Members, EP, F>,
    A: Aspect + In<E::Aspects, AP, E>,
    T: 'static,
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    FieldSelection {
        path,
        resolve: super::field_handle::checked_record_field::<F, E, A, T>,
        fragment: PhantomData,
    }
}

pub(super) mod sealed {
    pub trait Sealed {}
}
impl<F, E, A, T> sealed::Sealed for FieldSelection<F, E, A, T> {}
/// A field selected from an owned record. The constructor proves membership;
/// declaration or installed metadata proves the path and the exact value codec.
pub trait RecordSelection: sealed::Sealed + Copy + Send + Sync + 'static {
    type Entity;
    type Aspect;
    type Value: 'static;
    type Binding: crate::application_schema::ApplicationReadableScalarValueBinding<
        Value = Self::Value,
    >;
    /// Resolves declaration metadata; ordinary runtime access uses `installed`.
    fn checked(
        self,
    ) -> Result<
        RecordField<Self::Entity, Self::Aspect, Self::Value>,
        ApplicationSchemaDeclarationDenial,
    >;
    fn entity(self) -> &'static str;
    fn field(self) -> &'static str;
    fn aspect(self) -> &'static str;
    fn value_type_name(self) -> &'static str {
        <Self::Binding as crate::application_schema::ApplicationScalarValueBinding>::IDENTITY_NAME
    }
    fn unit(self) -> Option<&'static str> {
        <Self::Binding as crate::application_schema::ApplicationScalarValueBinding>::UNIT
            .map(|unit| unit.as_str())
    }
    fn scalar_family(self) -> worth_foundational::facade::ScalarAspectType {
        <Self::Binding as crate::application_schema::ApplicationScalarValueBinding>::SCALAR_FAMILY
    }
    fn expected_recipe(self) -> ApplicationFieldBindingRecipe;
    fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&RecordFieldDefinition, ApplicationSchemaDeclarationDenial>;
}
impl<F: Fragment, E: Entity, A: Aspect, T: 'static> RecordSelection for FieldSelection<F, E, A, T>
where
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    type Entity = E;
    type Aspect = A;
    type Value = T;
    type Binding = <F::Bindings as Scalar<T>>::Binding;
    fn checked(self) -> Result<RecordField<E, A, T>, ApplicationSchemaDeclarationDenial> {
        self.resolve()
    }
    fn entity(self) -> &'static str {
        FieldSelection::entity(self)
    }
    fn field(self) -> &'static str {
        FieldSelection::field(self)
    }
    fn aspect(self) -> &'static str {
        FieldSelection::aspect(self)
    }
    fn expected_recipe(self) -> ApplicationFieldBindingRecipe {
        self.binding_recipe()
    }
    fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&RecordFieldDefinition, ApplicationSchemaDeclarationDenial> {
        self.installed(catalog)
    }
}
