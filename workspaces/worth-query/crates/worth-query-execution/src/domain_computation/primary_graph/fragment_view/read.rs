//! Row construction stays typed by the fragment and the value marker.

use super::{FragmentDenial, ObservedEntity, ObservedTarget, Reading, View};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, BindingSet, CaseWitness, Entity, Fragment, FragmentDefinition, GroupWitness, In, OneOf,
    OptionalCaseWitness, OptionalGroupWitness, OptionalWitness, Record, RequiredWitness, Shape,
};

#[doc(hidden)]
pub trait Readable<Set: BindingSet, Members, Witness, E: Entity>: In<Members, Witness, E> {
    type Output;
    fn read(
        access: &mut Reading<'_>,
        definition: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial>;
}

impl<F: Fragment> View<'_, F, Reading<'_>> {
    pub fn read<E, Marker, EntityWitness, MemberWitness>(
        &mut self,
        target: &ObservedEntity<E>,
        _: Marker,
        work: &mut <F::Bindings as BindingSet>::Work<'_>,
    ) -> Result<Marker::Output, FragmentDenial>
    where
        E: Entity + In<F::Members, EntityWitness, F>,
        F::Bindings: BindingSet,
        Marker:
            In<E::Aspects, MemberWitness, E> + Readable<F::Bindings, E::Aspects, MemberWitness, E>,
    {
        Marker::read(&mut self.access, self.definition, target, work)
    }

    pub fn resolve<E, A, T, EntityWitness, AspectWitness>(
        &mut self,
        field: worth_query_declaration::facade::application_schema::fragment::RecordField<E, A, T>,
        value: T,
    ) -> Result<ObservedEntity<E>, FragmentDenial>
    where
        E: Entity + In<F::Members, EntityWitness, F>,
        A: Aspect + In<E::Aspects, AspectWitness, E>,
        T: 'static,
        F::Bindings: BindingSet,
    {
        let definition =
            self.definition
                .aspect::<E, A>()
                .ok_or(FragmentDenial::MemberNotInstalled {
                    entity: E::NAME,
                    member: A::NAME,
                })?;
        let installed = definition.fields().iter().find(|installed| {
            installed.recipe().locus() == field.locus()
                && installed.recipe().value_type()
                    == crate::equality_type_id::EqualityTypeId::of::<T>().into_external()
                && installed.recipe().binding_type() == field.recipe().binding_type()
                && installed.equality()
        });
        if installed.is_none() {
            return Err(FragmentDenial::InvalidField {
                path: format!("{}.{}", definition.path(), field.locus().field()),
            });
        }
        let value = field
            .encode(&value)
            .map_err(|denial| FragmentDenial::Encode {
                path: format!("{}.{}", definition.path(), field.locus().field()),
                cause: denial.cause().clone(),
            })?;
        Ok(ObservedEntity {
            erased: self.access.port.resolve(field.locus(), value)?,
            marker: PhantomData,
        })
    }

    pub fn target<E, Witness>(
        &self,
        target: &ObservedEntity<E>,
    ) -> Result<ObservedTarget<E>, FragmentDenial>
    where
        E: Entity + In<F::Members, Witness, F>,
    {
        Ok(ObservedTarget {
            erased: self.access.port.target(&target.erased)?,
            marker: PhantomData,
        })
    }
}

impl<A, Set, Members, P, E> Readable<Set, Members, (P, RequiredWitness), E> for A
where
    E: Entity,
    A: Aspect + In<Members, (P, RequiredWitness), E>,
    Set: Record<A::Row>,
{
    type Output = A::Row;
    fn read(
        access: &mut Reading<'_>,
        fragment: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial> {
        let definition = fragment
            .aspect::<E, A>()
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: E::NAME,
                member: A::NAME,
            })?;
        let raw = access
            .port
            .record(&target.erased, definition)?
            .ok_or_else(|| FragmentDenial::MissingAspect {
                path: definition.path().to_owned(),
            })?;
        decode_record::<Set, A::Row>(access, &raw, definition.path(), work)
    }
}

impl<A, Set, Members, P, E> Readable<Set, Members, (P, OptionalWitness), E> for A
where
    E: Entity,
    A: Aspect + In<Members, (P, OptionalWitness), E>,
    Set: Record<A::Row>,
{
    type Output = Option<A::Row>;
    fn read(
        access: &mut Reading<'_>,
        fragment: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial> {
        let definition = fragment
            .aspect::<E, A>()
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: E::NAME,
                member: A::NAME,
            })?;
        access
            .port
            .record(&target.erased, definition)?
            .map(|raw| decode_record::<Set, A::Row>(access, &raw, definition.path(), work))
            .transpose()
    }
}

macro_rules! group_read {
    ($witness:ident, $output:ty) => {
        impl<G, Set, Members, P, E> Readable<Set, Members, (P, $witness), E> for G
        where
            E: Entity,
            Set: BindingSet,
            G: OneOf + In<Members, (P, $witness), E>,
        {
            type Output = $output;
            fn read(
                _: &mut Reading<'_>,
                _: &FragmentDefinition,
                _: &ObservedEntity<E>,
                _: &mut Set::Work<'_>,
            ) -> Result<Self::Output, FragmentDenial> {
                Err(FragmentDenial::OneOfReadUnavailable)
            }
        }
    };
}
macro_rules! case_read {
    ($witness:ident) => {
        impl<A, Set, Members, P, Inner, E> Readable<Set, Members, (P, $witness<Inner>), E> for A
        where
            E: Entity,
            Set: BindingSet,
            A: Aspect + In<Members, (P, $witness<Inner>), E>,
        {
            type Output = Option<A::Row>;
            fn read(
                _: &mut Reading<'_>,
                _: &FragmentDefinition,
                _: &ObservedEntity<E>,
                _: &mut Set::Work<'_>,
            ) -> Result<Self::Output, FragmentDenial> {
                Err(FragmentDenial::OneOfReadUnavailable)
            }
        }
    };
}
group_read!(GroupWitness, G::Row);
group_read!(OptionalGroupWitness, Option<G::Row>);
case_read!(CaseWitness);
case_read!(OptionalCaseWitness);

fn decode_record<Set: Record<T>, T: 'static>(
    access: &mut Reading<'_>,
    raw: &std::collections::BTreeMap<String, worth_foundational::facade::AspectValue>,
    path: &str,
    work: &mut Set::Work<'_>,
) -> Result<T, FragmentDenial> {
    let mut decoded = None;
    access.port.with_decode_admission(path, &mut |admission| {
        decoded = Some(Set::record().decode_admitted(raw, path, work, admission)?);
        Ok(())
    })?;
    Ok(decoded.expect("the original owner invokes its decode loan exactly once"))
}
