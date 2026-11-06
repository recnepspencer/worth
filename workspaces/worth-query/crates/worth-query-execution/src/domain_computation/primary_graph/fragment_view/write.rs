//! Checked record encoding precedes native candidate effect authoring.

use super::{CandidateEntity, FragmentDenial, ObservedTarget, View, Writing};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, BindingSet, Entity, Fragment, In, Record, Relation,
};

impl<F: Fragment> View<'_, F, Writing<'_>> {
    pub fn target<E, Witness>(
        &self,
        target: &ObservedTarget<E>,
    ) -> Result<CandidateEntity<E>, FragmentDenial>
    where
        E: Entity + In<F::Members, Witness, F>,
    {
        Ok(CandidateEntity {
            erased: self.access.port.target(&target.erased)?,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn create<E, Witness>(
        &mut self,
        _: E,
        key: &str,
    ) -> Result<CandidateEntity<E>, FragmentDenial>
    where
        E: Entity + In<F::Members, Witness, F>,
    {
        let name = self
            .definition
            .entity::<E>()
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: E::NAME,
                member: E::NAME,
            })?;
        Ok(CandidateEntity {
            erased: self.access.port.create(name, key)?,
            marker: PhantomData,
        })
    }

    #[inline(always)]
    pub fn write<E, A, EntityWitness, AspectWitness>(
        &mut self,
        target: &CandidateEntity<E>,
        _: A,
        value: &A::Row,
        _work: &mut <F::Bindings as BindingSet>::Work<'_>,
    ) -> Result<(), FragmentDenial>
    where
        E: Entity + In<F::Members, EntityWitness, F>,
        A: Aspect + In<E::Aspects, AspectWitness, E>,
        F::Bindings: Record<A::Row>,
    {
        let entity = crate::equality_type_id::EqualityTypeId::of::<E>().into_external();
        let aspect = crate::equality_type_id::EqualityTypeId::of::<A>().into_external();
        let definition = record_for_write(
            self.definition,
            &target.erased,
            entity,
            aspect,
            E::NAME,
            A::NAME,
        )?;
        let encoded = definition.encode::<F::Bindings, A::Row>(value)?;
        write_record(
            self.access.port,
            self.definition,
            &target.erased,
            (entity, aspect, E::NAME),
            definition,
            encoded,
        )
    }

    #[inline(always)]
    pub fn link<E, R, EntityWitness, RelationWitness>(
        &mut self,
        from: &CandidateEntity<E>,
        _: R,
        to: &CandidateEntity<R::To>,
    ) -> Result<(), FragmentDenial>
    where
        E: Entity + In<F::Members, EntityWitness, F>,
        R: Relation + In<E::Relations, RelationWitness, E>,
    {
        let definition =
            self.definition
                .relation::<E, R>()
                .ok_or(FragmentDenial::MemberNotInstalled {
                    entity: E::NAME,
                    member: R::NAME,
                })?;
        self.access.port.link(&from.erased, definition, &to.erased)
    }
}

fn record_for_write<'a>(
    fragment: &'a worth_query_declaration::facade::application_schema::fragment::FragmentDefinition,
    target: &super::ErasedCandidate,
    entity: std::any::TypeId,
    aspect: std::any::TypeId,
    entity_name: &'static str,
    aspect_name: &'static str,
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
    if target.created_effect.is_none()
        && let Some(field) = definition.fields().iter().find(|field| !field.writable())
    {
        return Err(FragmentDenial::ReadOnly {
            path: field.path().to_owned(),
        });
    }
    Ok(definition)
}
fn write_record(
    port: &mut dyn super::WritePort,
    fragment: &worth_query_declaration::facade::application_schema::fragment::FragmentDefinition,
    target: &super::ErasedCandidate,
    member: (std::any::TypeId, std::any::TypeId, &'static str),
    definition: &worth_query_declaration::facade::application_schema::fragment::AspectDefinition,
    encoded: std::collections::BTreeMap<String, worth_foundational::facade::AspectValue>,
) -> Result<(), FragmentDenial> {
    if let Some(group) = fragment.case_group_by_identity(member.0, member.1) {
        let cases = group
            .cases()
            .iter()
            .map(|case| {
                fragment.case_aspect_by_identity(member.0, case).ok_or(
                    FragmentDenial::MemberNotInstalled {
                        entity: member.2,
                        member: case.name(),
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        return port.case_record(target, definition, group, &cases, encoded);
    }
    port.record(target, definition, encoded)
}
