//! Relation field handles come from the payload's record, with inferred membership.
use super::field_selection::{sealed, RecordSelection};
use super::{Aspect, Entity, Fragment, In, PayloadMembers, Record, RecordField, Relation, Scalar};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationFieldBindingRecipe as Recipe,
    ApplicationSchemaDeclarationDenial as Denial,
};
use std::{any::TypeId, marker::PhantomData};

pub struct RelationFieldSelection<F, E, R, A, T> {
    path: &'static str,
    marker: PhantomData<fn() -> (F, E, R, A, T)>,
}
impl<F, E, R, A, T> Copy for RelationFieldSelection<F, E, R, A, T> {}
impl<F, E, R, A, T> Clone for RelationFieldSelection<F, E, R, A, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<F, E, R, A, T> sealed::Sealed for RelationFieldSelection<F, E, R, A, T> {}

pub fn select_relation_field<F, E, R, A, T, EP, RP, AP>(
    _: F,
    _: E,
    _: R,
    _: A,
    path: &'static str,
) -> RelationFieldSelection<F, E, R, A, T>
where
    F: Fragment,
    E: Entity + In<F::Members, EP, F>,
    R: Relation + In<E::Relations, RP, E>,
    E::Relations: PayloadMembers<R, RP>,
    A: Aspect + In<<E::Relations as PayloadMembers<R, RP>>::Aspects, AP, R>,
    T: 'static,
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    RelationFieldSelection {
        path,
        marker: PhantomData,
    }
}
impl<F: Fragment, E: Entity, R: Relation, A: Aspect, T: 'static> RecordSelection
    for RelationFieldSelection<F, E, R, A, T>
where
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    type Entity = R;
    type Aspect = A;
    type Value = T;
    type Binding = <F::Bindings as Scalar<T>>::Binding;
    fn entity(self) -> &'static str {
        R::NAME
    }
    fn aspect(self) -> &'static str {
        super::identity::native_relation_aspect_key::<R, A>()
    }
    fn field(self) -> &'static str {
        self.path
    }
    fn expected_recipe(self) -> Recipe {
        Recipe::of::<Self::Binding>(Locus::for_relation(R::NAME, self.aspect(), self.path))
    }
    fn checked(self) -> Result<RecordField<R, A, T>, Denial> {
        super::field_handle::checked_field::<F::Bindings, R, A, T>(
            self.path,
            Locus::for_relation(R::NAME, self.aspect(), self.path),
            &format!("{}.{}.{}.{}", F::NAME, E::NAME, R::NAME, A::NAME),
        )
    }
    #[inline(always)]
    fn identity(self) -> super::ColumnIdentity {
        super::ColumnIdentity {
            fragment: TypeId::of::<F>(),
            record: TypeId::of::<R>(),
            aspect: TypeId::of::<A>(),
            value: TypeId::of::<T>(),
            binding: TypeId::of::<Self::Binding>(),
            fragment_name: F::NAME,
            record_name: R::NAME,
            aspect_name: A::NAME,
            native_aspect: self.aspect(),
            field: self.path,
            owner: crate::application_schema::ApplicationFieldBindingOwnerKind::Relation,
        }
    }
}

/// The payload selector derives its endpoints from its owning relation membership.
pub trait RelationSelection: RecordSelection {
    type From;
    type To;
    fn from(self) -> &'static str;
    fn to(self) -> &'static str;
}
impl<F: Fragment, E: Entity, R: Relation, A: Aspect, T: 'static> RelationSelection
    for RelationFieldSelection<F, E, R, A, T>
where
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    type From = E;
    type To = R::To;
    fn from(self) -> &'static str {
        E::NAME
    }
    fn to(self) -> &'static str {
        <R::To as Entity>::NAME
    }
}

impl<F: Fragment, E: Entity, R: Relation, A: Aspect, T: 'static>
    RelationFieldSelection<F, E, R, A, T>
where
    F::Bindings: Scalar<T> + Record<A::Row>,
{
    pub const fn at<EP, RP, AP>(path: &'static str) -> Self
    where
        E: In<F::Members, EP, F>,
        R: In<E::Relations, RP, E>,
        E::Relations: PayloadMembers<R, RP>,
        A: In<<E::Relations as PayloadMembers<R, RP>>::Aspects, AP, R>,
    {
        Self {
            path,
            marker: PhantomData,
        }
    }
}
