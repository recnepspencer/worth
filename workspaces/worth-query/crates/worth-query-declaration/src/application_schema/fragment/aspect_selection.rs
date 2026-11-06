//! A value selector proves the aspect's top-down membership without restating its contract.
use super::{Aspect, AspectDefinition, Entity, Fragment, FragmentCatalog, In};
use std::marker::PhantomData;
pub struct AspectSelection<F, E, A>(PhantomData<fn() -> (F, E, A)>);
impl<F, E, A> Copy for AspectSelection<F, E, A> {}
impl<F, E, A> Clone for AspectSelection<F, E, A> {
    fn clone(&self) -> Self {
        *self
    }
}
pub fn aspect<F, E, A, EP, AP>(_: F, _: E, _: A) -> AspectSelection<F, E, A>
where
    F: Fragment,
    E: Entity + In<F::Members, EP, F>,
    A: Aspect + In<E::Aspects, AP, E>,
{
    AspectSelection(PhantomData)
}
impl<F: Fragment, E: Entity, A: Aspect> AspectSelection<F, E, A> {
    pub fn entity(self) -> &'static str {
        E::NAME
    }
    pub fn name(self) -> &'static str {
        super::identity::native_aspect_key::<E, A>()
    }
    pub fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&AspectDefinition, super::super::ApplicationSchemaDeclarationDenial> {
        catalog
            .fragment_type::<F>()
            .and_then(|fragment| fragment.aspect::<E, A>())
            .ok_or_else(
                || super::super::ApplicationSchemaDeclarationDenial::InvalidRecordMember {
                    path: format!("{}.{}.{}", F::NAME, E::NAME, A::NAME),
                },
            )
    }
}

mod sealed {
    pub trait Sealed {}
}
/// An entity or relation aspect selected from its real top-down owner.
pub trait AspectSelector: sealed::Sealed + Copy {
    type Owner;
    fn owner(self) -> &'static str;
    fn name(self) -> &'static str;
    fn owner_kind(self) -> super::super::ApplicationFieldBindingOwnerKind;
    fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&AspectDefinition, super::super::ApplicationSchemaDeclarationDenial>;
}
impl<F, E, A> sealed::Sealed for AspectSelection<F, E, A> {}
impl<F: Fragment, E: Entity, A: Aspect> AspectSelector for AspectSelection<F, E, A> {
    type Owner = E;
    fn owner(self) -> &'static str {
        self.entity()
    }
    fn name(self) -> &'static str {
        self.name()
    }
    fn owner_kind(self) -> super::super::ApplicationFieldBindingOwnerKind {
        super::super::ApplicationFieldBindingOwnerKind::Entity
    }
    fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&AspectDefinition, super::super::ApplicationSchemaDeclarationDenial> {
        self.installed(catalog)
    }
}
pub struct RelationAspectSelection<F, E, R, A>(PhantomData<fn() -> (F, E, R, A)>);
impl<F, E, R, A> Copy for RelationAspectSelection<F, E, R, A> {}
impl<F, E, R, A> Clone for RelationAspectSelection<F, E, R, A> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<F, E, R, A> sealed::Sealed for RelationAspectSelection<F, E, R, A> {}
pub fn relation_aspect<F, E, R, A, EP, RP, AP>(
    _: F,
    _: E,
    _: R,
    _: A,
) -> RelationAspectSelection<F, E, R, A>
where
    F: Fragment,
    E: Entity + In<F::Members, EP, F>,
    R: super::Relation + In<E::Relations, RP, E>,
    E::Relations: super::PayloadMembers<R, RP>,
    A: Aspect + In<<E::Relations as super::PayloadMembers<R, RP>>::Aspects, AP, R>,
{
    RelationAspectSelection(PhantomData)
}
impl<F: Fragment, E: Entity, R: super::Relation, A: Aspect> AspectSelector
    for RelationAspectSelection<F, E, R, A>
{
    type Owner = R;
    fn owner(self) -> &'static str {
        R::NAME
    }
    fn name(self) -> &'static str {
        super::identity::native_relation_aspect_key::<R, A>()
    }
    fn owner_kind(self) -> super::super::ApplicationFieldBindingOwnerKind {
        super::super::ApplicationFieldBindingOwnerKind::Relation
    }
    fn installed(
        self,
        catalog: &FragmentCatalog,
    ) -> Result<&AspectDefinition, super::super::ApplicationSchemaDeclarationDenial> {
        catalog
            .fragment_type::<F>()
            .and_then(|f| f.relation_aspect::<R, A>())
            .ok_or_else(
                || super::super::ApplicationSchemaDeclarationDenial::InvalidRecordMember {
                    path: format!("{}.{}.{}.{}", F::NAME, E::NAME, R::NAME, A::NAME),
                },
            )
    }
}
