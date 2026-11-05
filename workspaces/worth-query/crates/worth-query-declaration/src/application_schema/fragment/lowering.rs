//! Tuple traversal extracts typed facts; concrete emission owns flat schema meaning.

use super::super::{
    ApplicationSchemaDeclarationBuilder as Builder, ApplicationSchemaDeclarationDenial as Denial,
};
use super::declaration::{Aspect, Entity, Fragment, Registration, Relation};
use super::identity::Names;
use super::record::{Record, Shape};
use std::marker::PhantomData;

mod aspect;

#[doc(hidden)]
#[derive(Default)]
pub struct Lowering {
    names: Names,
    units: std::collections::BTreeSet<String>,
}

impl Lowering {
    pub(super) fn root(name: &str) -> Result<Self, Denial> {
        let mut state = Self::default();
        state.names.insert(name)?;
        Ok(state)
    }
}

#[doc(hidden)]
pub struct EntityWitness<A, R>(PhantomData<(A, R)>);
#[doc(hidden)]
pub struct FragmentWitness<M>(PhantomData<M>);
#[doc(hidden)]
pub struct RegistrationWitness;
#[doc(hidden)]
pub struct RequiredWitness;
#[doc(hidden)]
pub struct OptionalWitness;

#[doc(hidden)]
pub trait FragmentMember<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
}

impl<S, Set, E, A, R> FragmentMember<S, Set, EntityWitness<A, R>> for E
where
    E: Entity,
    E::Aspects: AspectMembers<S, Set, A>,
    E::Relations: RelationMembers<S, R>,
{
    fn lower(
        mut builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let path = format!("{prefix}.{}", E::NAME);
        state.names.insert(&path)?;
        builder
            .member_provenance
            .fragments
            .catalog
            .entity::<E>(&path)?;
        builder.member_provenance.register_entity::<E>(E::NAME);
        builder.push_member_in_place(super::super::ApplicationSchemaMember::Entity {
            entity: E::NAME.to_owned(),
        });
        let builder = E::Aspects::lower(builder, &path, E::NAME, state)?;
        E::Relations::lower(builder, &path, E::NAME, state)
    }
}

impl<S, Set, F, W> FragmentMember<S, Set, FragmentWitness<W>> for F
where
    F: Fragment,
    F::Members: FragmentMembers<S, F::Bindings, W>,
{
    fn lower(
        mut builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let path = format!("{prefix}.{}", F::NAME);
        state.names.insert(&path)?;
        builder
            .member_provenance
            .fragments
            .catalog
            .register::<F>(&path)?;
        F::Members::lower(builder, &path, state)
    }
}

impl<S, Set, T: Registration<S>> FragmentMember<S, Set, RegistrationWitness> for T {
    fn lower(builder: Builder<S>, _: &str, _: &mut Lowering) -> Result<Builder<S>, Denial> {
        T::register(builder)
    }
}

#[doc(hidden)]
pub trait AspectMember<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
}

impl<S, Set, A> AspectMember<S, Set, RequiredWitness> for A
where
    A: Aspect,
    Set: Record<A::Row>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        lower_aspect::<S, Set, A>(builder, prefix, entity, false, None, state)
    }
}

impl<S, Set, A> AspectMember<S, Set, OptionalWitness> for Option<A>
where
    A: Aspect,
    Set: Record<A::Row>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        lower_aspect::<S, Set, A>(builder, prefix, entity, true, None, state)
    }
}

#[doc(hidden)]
pub trait FragmentMembers<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
}
#[doc(hidden)]
pub trait AspectMembers<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
}
#[doc(hidden)]
pub trait RelationMembers<Schema, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
}

impl<S, Set> FragmentMembers<S, Set, ()> for () {
    fn lower(builder: Builder<S>, _: &str, _: &mut Lowering) -> Result<Builder<S>, Denial> {
        Ok(builder)
    }
}
impl<S, Set> AspectMembers<S, Set, ()> for () {
    fn lower(
        builder: Builder<S>,
        _: &str,
        _: &str,
        _: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        Ok(builder)
    }
}
impl<S> RelationMembers<S, ()> for () {
    fn lower(
        builder: Builder<S>,
        _: &str,
        _: &str,
        _: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        Ok(builder)
    }
}

macro_rules! tuples {
    ($($member:ident $witness:ident),+) => {
        impl<S, Set, $($member, $witness),+> FragmentMembers<S, Set, ($($witness,)+)> for ($($member,)+)
        where $($member: FragmentMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower(builder, prefix, state)?;)+ Ok(builder)
            }
        }
        impl<S, Set, $($member, $witness),+> AspectMembers<S, Set, ($($witness,)+)> for ($($member,)+)
        where $($member: AspectMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower(builder, prefix, entity, state)?;)+ Ok(builder)
            }
        }
    };
}

macro_rules! relations {
    ($($relation:ident),+) => {
        impl<S, $($relation: Relation),+> RelationMembers<S, ()> for ($($relation,)+) {
            fn lower(mut builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(state.names.insert(&format!("{prefix}.{}", $relation::NAME))?;
                builder.member_provenance.fragments.catalog.relation::<$relation>(prefix, entity)?;
                builder.push_member_in_place(super::super::ApplicationSchemaMember::Relation { relation: $relation::NAME.to_owned(), from: entity.to_owned(), to: <$relation::To as Entity>::NAME.to_owned(), integrity: super::declaration::relation_integrity::<$relation>().into() });)+
                Ok(builder)
            }
        }
    };
}

pub(super) fn lower_aspect<S, Set, A>(
    mut builder: Builder<S>,
    prefix: &str,
    entity: &str,
    optional: bool,
    group_contract: Option<&[u8; 32]>,
    state: &mut Lowering,
) -> Result<Builder<S>, Denial>
where
    A: Aspect,
    Set: Record<A::Row>,
{
    let shape = <Set as Record<A::Row>>::record();
    let columns = shape.columns("").map_err(|cause| Denial::InvalidRecord {
        path: format!("{prefix}.{}.{}", A::NAME, cause.path()),
    })?;
    builder
        .member_provenance
        .fragments
        .register_bindings(shape.bindings());
    let definition = builder
        .member_provenance
        .fragments
        .catalog
        .aspect::<Set, A>(prefix, entity, &columns, optional)?;
    aspect::emit(builder, &definition, group_contract, state)
}

tuples!(A WA);
tuples!(A WA, B WB);
tuples!(A WA, B WB, C WC);
tuples!(A WA, B WB, C WC, D WD);
tuples!(A WA, B WB, C WC, D WD, E WE);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
tuples!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
relations!(A);
relations!(A, B);
relations!(A, B, C);
relations!(A, B, C, D);
relations!(A, B, C, D, E);
relations!(A, B, C, D, E, F);
relations!(A, B, C, D, E, F, G);
relations!(A, B, C, D, E, F, G, H);
