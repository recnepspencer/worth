//! Row construction stays typed by the fragment and the value marker.

use super::{FragmentDenial, ObservedEntity, ObservedTarget, Reading, View};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, BindingSet, Entity, Fragment, FragmentDefinition, In, OptionalWitness, Record,
    RequiredWitness, Shape,
};

#[doc(hidden)]
pub trait Readable<Set: BindingSet, Kind, E: Entity> {
    type Output;
    fn read(
        access: &mut Reading<'_>,
        definition: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial>;
}

impl<F: Fragment> View<'_, F, Reading<'_>> {
    #[inline(always)]
    pub fn read<E, Marker, EntityWitness, MemberWitness>(
        &mut self,
        target: &ObservedEntity<E>,
        _: Marker,
        work: &mut <F::Bindings as BindingSet>::Work<'_>,
    ) -> Result<Marker::Output, FragmentDenial>
    where
        E: Entity,
        F::Bindings: BindingSet,
        Marker: In<F::Members, (EntityWitness, MemberWitness), (F, E)>
            + Readable<F::Bindings, MemberWitness, E>,
    {
        Marker::read(&mut self.access, self.definition, target, work)
    }

    pub fn resolve<E, A, T, EntityWitness, AspectWitness>(
        &mut self,
        field: worth_query_declaration::facade::application_schema::fragment::RecordField<E, A, T>,
        value: T,
    ) -> Result<ObservedEntity<E>, FragmentDenial>
    where
        E: Entity,
        A: Aspect + In<F::Members, (EntityWitness, AspectWitness), (F, E)>,
        T: 'static,
        F::Bindings: BindingSet,
    {
        let definition = resolve_definition(
            self.definition,
            crate::equality_type_id::EqualityTypeId::of::<E>().into_external(),
            crate::equality_type_id::EqualityTypeId::of::<A>().into_external(),
            E::NAME,
            A::NAME,
            field.recipe(),
        )?;
        let value = encoded_resolution(definition, field.locus(), field.encode(&value))?;
        Ok(ObservedEntity {
            erased: self.access.port.resolve(field.locus(), value)?,
            marker: PhantomData,
        })
    }

    pub fn target<E>(&self, target: &ObservedEntity<E>) -> Result<ObservedTarget<E>, FragmentDenial>
    where
        E: Entity,
    {
        Ok(ObservedTarget {
            erased: self.access.port.target(&target.erased)?,
            marker: PhantomData,
        })
    }
}

impl<A, Set, E> Readable<Set, RequiredWitness, E> for A
where
    E: Entity,
    A: Aspect,
    Set: Record<A::Row>,
{
    type Output = A::Row;
    fn read(
        access: &mut Reading<'_>,
        fragment: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial> {
        let (definition, raw) = read_record(
            access,
            fragment,
            &target.erased,
            crate::equality_type_id::EqualityTypeId::of::<E>().into_external(),
            crate::equality_type_id::EqualityTypeId::of::<A>().into_external(),
            (E::NAME, A::NAME),
            true,
        )?;
        decode_record::<Set, A::Row>(
            access,
            &raw.expect("required presence was checked"),
            definition.path(),
            work,
        )
    }
}

impl<A, Set, E> Readable<Set, OptionalWitness, E> for A
where
    E: Entity,
    A: Aspect,
    Set: Record<A::Row>,
{
    type Output = Option<A::Row>;
    fn read(
        access: &mut Reading<'_>,
        fragment: &FragmentDefinition,
        target: &ObservedEntity<E>,
        work: &mut Set::Work<'_>,
    ) -> Result<Self::Output, FragmentDenial> {
        let (definition, raw) = read_record(
            access,
            fragment,
            &target.erased,
            crate::equality_type_id::EqualityTypeId::of::<E>().into_external(),
            crate::equality_type_id::EqualityTypeId::of::<A>().into_external(),
            (E::NAME, A::NAME),
            false,
        )?;
        raw.map(|raw| decode_record::<Set, A::Row>(access, &raw, definition.path(), work))
            .transpose()
    }
}

pub(super) fn decode_record<Set: Record<T>, T: 'static>(
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

type RawRecord = std::collections::BTreeMap<String, worth_foundational::facade::AspectValue>;
fn read_record<'a>(
    access: &mut Reading<'_>,
    fragment: &'a FragmentDefinition,
    target: &super::ErasedObserved,
    entity: std::any::TypeId,
    aspect: std::any::TypeId,
    names: (&'static str, &'static str),
    required: bool,
) -> Result<
    (
        &'a worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
        Option<RawRecord>,
    ),
    FragmentDenial,
> {
    let definition =
        fragment
            .aspect_by_identity(entity, aspect)
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: names.0,
                member: names.1,
            })?;
    let raw = access.port.record(target, definition)?;
    if required && raw.is_none() {
        return Err(FragmentDenial::MissingAspect {
            path: definition.path().to_owned(),
        });
    }
    Ok((definition, raw))
}
fn resolve_definition<'a>(
    fragment: &'a FragmentDefinition,
    entity: std::any::TypeId,
    aspect: std::any::TypeId,
    entity_name: &'static str,
    aspect_name: &'static str,
    recipe: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingRecipe,
) -> Result<
    &'a worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
    FragmentDenial,
> {
    let definition =
        fragment
            .aspect_by_identity(entity, aspect)
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: entity_name,
                member: aspect_name,
            })?;
    if !definition.fields().iter().any(|installed| {
        installed.recipe().locus() == recipe.locus()
            && installed.recipe().value_type() == recipe.value_type()
            && installed.recipe().binding_type() == recipe.binding_type()
            && installed.equality()
    }) {
        return Err(FragmentDenial::InvalidField {
            path: format!("{}.{}", definition.path(), recipe.locus().field()),
        });
    }
    Ok(definition)
}
fn encoded_resolution(
    definition: &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
    field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
    value: Result<
        worth_foundational::facade::AspectValue,
        worth_query_declaration::facade::application_schema::fragment::FieldEncodeRefusal,
    >,
) -> Result<worth_foundational::facade::AspectValue, FragmentDenial> {
    value.map_err(|denial| FragmentDenial::Encode {
        path: format!("{}.{}", definition.path(), field.field()),
        cause: denial.cause().clone(),
    })
}
