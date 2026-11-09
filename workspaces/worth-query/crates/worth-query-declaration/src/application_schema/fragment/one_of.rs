//! One case list owns every exclusive case and its compiler-visible membership.

use super::super::ApplicationValue;
use super::super::{
    ApplicationSchemaDeclarationBuilder as Builder, ApplicationSchemaDeclarationDenial as Denial,
};
use super::lowering::{lower_aspect, AspectMember, Lowering};
use super::{Aspect, BindingSet, Record, RecordRefusal, RecordRefusalKind, Shape};
use std::collections::BTreeMap;
use std::marker::PhantomData;
mod definition;
mod join;
pub use definition::{CaseDefinition, GroupDefinition};
pub use join::JoinCases;

pub trait OneOf: 'static {
    type Row: 'static;
    type Cases;
}

/// Used in Entity::Aspects; the one-of's own unit marker is passed to reads.
pub struct Exclusive<C>(PhantomData<C>);

#[doc(hidden)]
pub struct GroupWitness;
#[doc(hidden)]
pub struct OptionalGroupWitness;
#[doc(hidden)]
pub struct CaseWitness<P>(PhantomData<P>);
#[doc(hidden)]
pub struct OptionalCaseWitness<P>(PhantomData<P>);

#[doc(hidden)]
pub trait CaseRows<Set: BindingSet, Enum> {
    fn contains(name: &str) -> bool;
    fn decode(
        name: &str,
        values: &BTreeMap<String, ApplicationValue>,
        prefix: &str,
        work: &mut Set::Work<'_>,
        admission: &mut super::super::ValueDecodeAdmission<'_>,
    ) -> Result<Enum, RecordRefusal>;
}

#[doc(hidden)]
pub trait Cases<S, Set: BindingSet, Enum>: CaseRows<Set, Enum> {
    fn definitions() -> Vec<CaseDefinition>;
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
        contract: &[u8; 32],
    ) -> Result<Builder<S>, Denial>;
}

impl<S, Set, C> AspectMember<S, Set, GroupWitness> for Exclusive<C>
where
    C: OneOf,
    Set: BindingSet,
    C::Cases: Cases<S, Set, C::Row>,
{
    fn lower(
        mut builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let group = GroupDefinition::of::<C>(C::Cases::definitions(), false);
        builder
            .member_provenance
            .fragments
            .catalog
            .group(prefix, entity, group.clone())?;
        C::Cases::lower(builder, prefix, entity, state, group.contract())
    }
}

impl<S, Set, C> AspectMember<S, Set, OptionalGroupWitness> for Option<Exclusive<C>>
where
    C: OneOf,
    Set: BindingSet,
    C::Cases: Cases<S, Set, C::Row>,
{
    fn lower(
        mut builder: Builder<S>,
        prefix: &str,
        entity: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let group = GroupDefinition::of::<C>(C::Cases::definitions(), true);
        builder
            .member_provenance
            .fragments
            .catalog
            .group(prefix, entity, group.clone())?;
        C::Cases::lower(builder, prefix, entity, state, group.contract())
    }
}

macro_rules! cases {
    ($($case:ident),+) => {
        impl<Set, Enum, $($case),+> CaseRows<Set, Enum> for ($($case,)+)
        where $($case: Aspect, Set: Record<$case::Row>, Enum: From<$case::Row> + 'static,)+ {
            fn contains(name: &str) -> bool { false $(|| name == $case::NAME)+ }
            fn decode(name: &str, values: &BTreeMap<String, ApplicationValue>, prefix: &str, work: &mut Set::Work<'_>, admission: &mut super::super::ValueDecodeAdmission<'_>) -> Result<Enum, RecordRefusal> {
                $(if name == $case::NAME { return <Set as Record<$case::Row>>::record().decode_admitted(values, prefix, work, admission).map(Enum::from); })+
                Err(RecordRefusal::new(prefix, RecordRefusalKind::UnknownCase))
            }
        }
        impl<S, Set, Enum, $($case),+> Cases<S, Set, Enum> for ($($case,)+)
        where $($case: Aspect, Set: Record<$case::Row>, Enum: From<$case::Row> + 'static,)+ {
            fn definitions() -> Vec<CaseDefinition> { vec![$(CaseDefinition::of::<Set, $case>(),)+] }
            fn lower(builder: Builder<S>, prefix: &str, entity: &str, state: &mut Lowering, contract: &[u8; 32]) -> Result<Builder<S>, Denial> {
                $(let builder = lower_aspect::<S, Set, $case>(builder, prefix, entity, true, Some(contract), state)?;)+
                Ok(builder)
            }
        }
    };
}

cases!(A);
cases!(A, B);
cases!(A, B, C);
cases!(A, B, C, D);
cases!(A, B, C, D, E);
cases!(A, B, C, D, E, F);
cases!(A, B, C, D, E, F, G);
cases!(A, B, C, D, E, F, G, H);
