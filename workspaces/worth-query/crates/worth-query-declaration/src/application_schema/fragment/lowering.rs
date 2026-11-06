//! Tuple traversal extracts typed facts; concrete emission owns flat schema meaning.

use super::super::{
    ApplicationSchemaDeclarationBuilder as Builder, ApplicationSchemaDeclarationDenial as Denial,
};
use super::declaration::{Aspect, Entity, Fragment, Registration};
use super::identity::Names;
use super::record::{Record, Shape};
use std::marker::PhantomData;

mod aspect;
mod contribution;
mod correction;
mod relation_payload;

#[doc(hidden)]
#[derive(Default)]
pub struct Lowering {
    names: Names,
    units: std::collections::BTreeSet<String>,
    content_formats: std::collections::BTreeSet<(std::any::TypeId, [u8; 32])>,
    registrations: std::collections::BTreeSet<(String, std::any::TypeId)>,
    contributions: std::collections::BTreeSet<
        crate::application_schema::ApplicationSchemaContributionIdentity,
    >,
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
pub struct NestedWitness<W>(PhantomData<W>);
#[doc(hidden)]
pub struct RequiredWitness;
#[doc(hidden)]
pub struct OptionalWitness;

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "member `{Self}` cannot be declared in schema `{Schema}`",
    note = "implement the member's schema requirements (including its schema binding trait) for `{Schema}`"
)]
pub trait FragmentMember<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        Schema: crate::application_schema::ApplicationSchema,
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<Schema>,
    {
        let _ = source;
        Err(visitor)
    }
}

impl<S, Set, E, A, R> FragmentMember<S, Set, EntityWitness<A, R>> for E
where
    E: Entity,
    E::Aspects: AspectMembers<S, Set, A>,
    E::Relations: RelationMembers<S, Set, R>,
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
        let builder =
            <E::Aspects as AspectMembers<S, Set, A>>::lower(builder, &path, E::NAME, state)?;
        <E::Relations as RelationMembers<S, Set, R>>::lower(builder, &path, E::NAME, state)
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
        super::super::identifier_validation::validate_simple_identifier(F::NAME)?;
        let path = format!("{prefix}.{}", F::NAME);
        state.names.insert(&path)?;
        builder
            .member_provenance
            .fragments
            .catalog
            .register::<F>(&path)?;
        <F::Members as FragmentMembers<S, F::Bindings, W>>::lower(builder, &path, state)
    }
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        S: crate::application_schema::ApplicationSchema,
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<S>,
    {
        <F::Members as FragmentMembers<S, F::Bindings, W>>::visit_correction(source, visitor)
    }
}

impl<S, Set, T: Registration<S>> FragmentMember<S, Set, RegistrationWitness> for T {
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        if !state
            .registrations
            .insert((prefix.to_owned(), std::any::TypeId::of::<T>()))
        {
            return Err(Denial::DuplicateFragmentMember {
                path: format!("{prefix}.{}", std::any::type_name::<T>()),
            });
        }
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
#[diagnostic::on_unimplemented(
    message = "members `{Self}` cannot be declared in schema `{Schema}`",
    note = "each member's schema requirements must be implemented by `{Schema}`"
)]
pub trait FragmentMembers<Schema, Set, Witness> {
    fn lower(
        builder: Builder<Schema>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<Schema>, Denial>;
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        Schema: crate::application_schema::ApplicationSchema,
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<Schema>,
    {
        let _ = source;
        Err(visitor)
    }
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
pub trait RelationMembers<Schema, Set, Witness> {
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
impl<S, Set> RelationMembers<S, Set, ()> for () {
    fn lower(
        builder: Builder<S>,
        _: &str,
        _: &str,
        _: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        Ok(builder)
    }
}

impl<S, Set, Members, W> FragmentMember<S, Set, NestedWitness<W>> for Members
where
    Members: FragmentMembers<S, Set, W>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        <Members as FragmentMembers<S, Set, W>>::lower(builder, prefix, state)
    }
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        S: crate::application_schema::ApplicationSchema,
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<S>,
    {
        <Members as FragmentMembers<S, Set, W>>::visit_correction(source, visitor)
    }
}
impl<S, Set, Members, W> AspectMember<S, Set, NestedWitness<W>> for Members
where
    Members: AspectMembers<S, Set, W>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        <Members as AspectMembers<S, Set, W>>::lower(builder, prefix, entity, state)
    }
}

macro_rules! tuples {
    ($($member:ident $witness:ident),+) => {
        impl<S, Set, $($member, $witness),+> FragmentMembers<S, Set, ($($witness,)+)> for ($($member,)+)
        where $($member: FragmentMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $member::lower(builder, prefix, state)?;)+ Ok(builder)
            }
            fn visit_correction<Visitor>(source: std::any::TypeId, visitor: Visitor) -> Result<Visitor::Output, Visitor>
            where S: crate::application_schema::ApplicationSchema, Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<S> {
                $(let visitor = match <$member as FragmentMember<S, Set, $witness>>::visit_correction(source, visitor) {
                    Ok(output) => return Ok(output), Err(visitor) => visitor,
                };)+
                Err(visitor)
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
    ($($relation:ident $witness:ident),+) => {
        impl<S, Set, $($relation, $witness),+> RelationMembers<S, Set, ($($witness,)+)> for ($($relation,)+)
        where $($relation: relation_payload::RelationMember<S, Set, $witness>,)+ {
            fn lower(builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering) -> Result<Builder<S>, Denial> {
                $(let builder = $relation::lower(builder, prefix, entity, state)?;)+ Ok(builder)
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
relations!(A WA);
relations!(A WA, B WB);
relations!(A WA, B WB, C WC);
relations!(A WA, B WB, C WC, D WD);
relations!(A WA, B WB, C WC, D WD, E WE);
relations!(A WA, B WB, C WC, D WD, E WE, F WF);
relations!(A WA, B WB, C WC, D WD, E WE, F WF, G WG);
relations!(A WA, B WB, C WC, D WD, E WE, F WF, G WG, H WH);
