//! Native parent presence certifies the selected case before any row decoding.

use super::read::{decode_record, Readable};
use super::{FragmentDenial, ObservedEntity, Reading};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, AspectDefinition, BindingSet, CaseRows, CaseWitness, Entity, FragmentDefinition,
    GroupDefinition, GroupWitness, In, OneOf, OptionalCaseWitness, OptionalGroupWitness, Record,
};

type Selected<'a> = Option<(&'a AspectDefinition, BTreeMap<String, AspectValue>)>;

fn observe_group<'a, E: Entity>(
    access: &mut Reading<'_>,
    fragment: &'a FragmentDefinition,
    target: &ObservedEntity<E>,
    group: &GroupDefinition,
) -> Result<Selected<'a>, FragmentDenial> {
    let mut selected = None;
    let mut present = Vec::new();
    for case in group.cases() {
        let aspect = fragment
            .case_aspect::<E>(case)
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: E::NAME,
                member: case.name(),
            })?;
        if let Some(raw) = access.port.record(&target.erased, aspect)? {
            present.push(aspect.path().to_owned());
            if selected.is_none() {
                selected = Some((aspect, raw));
            }
        }
    }
    if present.len() > 1 || (present.is_empty() && !group.optional()) {
        return Err(FragmentDenial::CorruptOneOf {
            entity: E::NAME,
            group: group.name(),
            present,
        });
    }
    Ok(selected)
}

fn read_group<Set, G, E>(
    access: &mut Reading<'_>,
    fragment: &FragmentDefinition,
    target: &ObservedEntity<E>,
    work: &mut Set::Work<'_>,
) -> Result<Option<G::Row>, FragmentDenial>
where
    E: Entity,
    Set: BindingSet,
    G: OneOf,
    G::Cases: CaseRows<Set, G::Row>,
{
    let group = fragment
        .group::<E, G>()
        .ok_or(FragmentDenial::MemberNotInstalled {
            entity: E::NAME,
            member: "one-of",
        })?;
    let Some((aspect, raw)) = observe_group(access, fragment, target, group)? else {
        return Ok(None);
    };
    let mut decoded = None;
    access
        .port
        .with_decode_admission(aspect.path(), &mut |admission| {
            decoded = Some(G::Cases::decode(
                aspect.name(),
                &raw,
                aspect.path(),
                work,
                admission,
            )?);
            Ok(())
        })?;
    Ok(Some(decoded.expect(
        "the original owner invokes its decode loan exactly once",
    )))
}

macro_rules! group_read {
    ($witness:ident, $output:ty, $finish:expr) => {
        impl<G, Set, Members, P, E> Readable<Set, Members, (P, $witness), E> for G
        where
            E: Entity,
            Set: BindingSet,
            G: OneOf + In<Members, (P, $witness), E>,
            G::Cases: CaseRows<Set, G::Row>,
        {
            type Output = $output;
            fn read(
                access: &mut Reading<'_>,
                fragment: &FragmentDefinition,
                target: &ObservedEntity<E>,
                work: &mut Set::Work<'_>,
            ) -> Result<Self::Output, FragmentDenial> {
                let value = read_group::<Set, G, E>(access, fragment, target, work)?;
                ($finish)(value)
            }
        }
    };
}
group_read!(GroupWitness, G::Row, |value: Option<G::Row>| value
    .ok_or_else(|| {
        FragmentDenial::CorruptOneOf {
            entity: E::NAME,
            group: "one-of",
            present: Vec::new(),
        }
    }));
group_read!(OptionalGroupWitness, Option<G::Row>, Ok);

macro_rules! case_read {
    ($witness:ident) => {
        impl<A, Set, Members, P, Inner, E> Readable<Set, Members, (P, $witness<Inner>), E> for A
        where
            E: Entity,
            Set: Record<A::Row>,
            A: Aspect + In<Members, (P, $witness<Inner>), E>,
        {
            type Output = Option<A::Row>;
            fn read(
                access: &mut Reading<'_>,
                fragment: &FragmentDefinition,
                target: &ObservedEntity<E>,
                work: &mut Set::Work<'_>,
            ) -> Result<Self::Output, FragmentDenial> {
                let group =
                    fragment
                        .case_group::<E, A>()
                        .ok_or(FragmentDenial::MemberNotInstalled {
                            entity: E::NAME,
                            member: A::NAME,
                        })?;
                let Some((aspect, raw)) = observe_group(access, fragment, target, group)? else {
                    return Ok(None);
                };
                if aspect.name() != A::NAME {
                    return Ok(None);
                }
                decode_record::<Set, A::Row>(access, &raw, aspect.path(), work).map(Some)
            }
        }
    };
}
case_read!(CaseWitness);
case_read!(OptionalCaseWitness);
