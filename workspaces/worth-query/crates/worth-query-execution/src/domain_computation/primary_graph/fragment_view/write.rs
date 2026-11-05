//! Checked record encoding precedes native candidate effect authoring.

use super::{CandidateEntity, FragmentDenial, ObservedTarget, View, Writing};
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, BindingSet, Entity, Fragment, In, Record, Relation, Shape,
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
        let definition =
            self.definition
                .aspect::<E, A>()
                .ok_or(FragmentDenial::MemberNotInstalled {
                    entity: E::NAME,
                    member: A::NAME,
                })?;
        if target.erased.created_effect.is_none()
            && definition.fields().iter().any(|field| !field.writable())
        {
            return Err(FragmentDenial::ReadOnly {
                path: definition.path().to_owned(),
            });
        }
        let encoded = <F::Bindings as Record<A::Row>>::record().encode(value, definition.path())?;
        if let Some(group) = self.definition.case_group::<E, A>() {
            let cases = group
                .cases()
                .iter()
                .map(|case| {
                    self.definition.case_aspect::<E>(case).ok_or(
                        FragmentDenial::MemberNotInstalled {
                            entity: E::NAME,
                            member: case.name(),
                        },
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            return self.access.port.case_record(
                &target.erased,
                definition,
                group,
                &cases,
                encoded,
            );
        }
        self.access.port.record(&target.erased, definition, encoded)
    }

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
