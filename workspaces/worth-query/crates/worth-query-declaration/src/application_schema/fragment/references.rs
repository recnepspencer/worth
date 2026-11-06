//! Existing operation and capability references derive their names from members.
use super::{Entity, Fragment, In, Relation};
use crate::application_schema::{ApplicationEntityRef, ApplicationRelationRef};

pub fn entity<S, F, E, P>(_: F, _: E) -> ApplicationEntityRef<S, E>
where
    F: Fragment,
    E: Entity + In<F::Members, P, F>,
{
    ApplicationEntityRef::from_schema_identifier(E::NAME)
}

pub fn relation<S, F, E, R, EP, RP>(_: F, _: E, _: R) -> ApplicationRelationRef<S, R, E, R::To>
where
    F: Fragment,
    E: Entity,
    R: Relation + In<F::Members, (EP, RP), (F, E)>,
{
    ApplicationRelationRef::from_schema_identifiers(
        R::NAME,
        E::NAME,
        <R::To as Entity>::NAME,
        super::declaration::relation_integrity::<R>(),
    )
}
