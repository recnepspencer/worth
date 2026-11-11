//! Relation positions retain their source entity and payload membership.
use super::{private, In};
use crate::application_schema::fragment::{
    Entity, Extend, Fragment, OptionalExtensionMembers, PayloadMembers,
};
use std::marker::PhantomData;
#[doc(hidden)]
pub struct OwnedRelationPosition<E, EntityPosition, RelationPosition>(
    PhantomData<(E, EntityPosition, RelationPosition)>,
);
#[doc(hidden)]
pub struct ExtensionRelationPosition<P>(PhantomData<P>);
impl<R, F: Fragment, E: Entity, EP, P, Kind>
    private::Sealed<F::Members, (OwnedRelationPosition<E, EP, P>, Kind), (F, E)> for R
where
    E: In<F::Members, EP, F>,
    R: In<E::Relations, (P, Kind), E>,
{
}
impl<R, F, E: Entity, Members: OptionalExtensionMembers<E>, Relations, P, Kind>
    private::Sealed<Extend<E, Members, Relations>, (ExtensionRelationPosition<P>, Kind), (F, E)>
    for R
where
    R: In<Relations, (P, Kind), E>,
{
}
impl<Members, E: Entity, EP, P, Kind, R> PayloadMembers<R, (OwnedRelationPosition<E, EP, P>, Kind)>
    for Members
where
    E::Relations: PayloadMembers<R, (P, Kind)>,
{
    type Aspects = <E::Relations as PayloadMembers<R, (P, Kind)>>::Aspects;
}
impl<E: Entity, Members: OptionalExtensionMembers<E>, Relations, P, Kind, R>
    PayloadMembers<R, (ExtensionRelationPosition<P>, Kind)> for Extend<E, Members, Relations>
where
    Relations: PayloadMembers<R, (P, Kind)>,
{
    type Aspects = <Relations as PayloadMembers<R, (P, Kind)>>::Aspects;
}
