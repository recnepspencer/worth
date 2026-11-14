//! Exact disclosed pre-images retain the original Home or Memory source route.
use super::super::occurrence::CorrectionOccurrence;
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

pub(super) enum DisclosedPreImage<'a> {
    Retained(&'a CorrectionOccurrence),
    Captured(&'a WorthQueryPrimaryGraphCommittedApplication),
}
impl DisclosedPreImage<'_> {
    fn retained(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        match self {
            Self::Retained(value) => value.memory_preimage(),
            Self::Captured(value) => value.commit_evidence().retained_preimage(),
        }
    }
    fn outputs(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        match self {
            Self::Retained(value) => value.correspondence(),
            Self::Captured(value) => value.commit_evidence().output_correspondence(),
        }
    }
}
impl<S> ApplicationCorrectionPreImage<S> for DisclosedPreImage<'_> {
    fn field_state_for_output(
        &self,
        role: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor,
        locus: &DeclaredPreImageLocus<S>,
    ) -> worth_query_declaration::facade::application_aftermath::ApplicationCorrectionFieldPreImage<'_>{
        use worth_query_declaration::facade::application_aftermath::ApplicationCorrectionFieldPreImage as State;
        if role.entity() != locus.entity() {
            return State::Uncaptured;
        }
        let mut roles =
            self.outputs()
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let Some((_, _, _, original)) = roles.next() else {
            return State::Uncaptured;
        };
        if roles.next().is_some() {
            return State::Uncaptured;
        }
        let Some(aspect) = worth_foundational::facade::AspectKey::new(locus.aspect()) else {
            return State::Uncaptured;
        };
        let Some(field) = worth_foundational::facade::FieldKey::new(locus.field()) else {
            return State::Uncaptured;
        };
        let locator = worth_foundational::facade::AspectFieldLocator::new(
            worth_foundational::facade::LocatorAuthority::Planned,
            aspect,
            worth_foundational::facade::CanonicalFieldPath::single(field),
        );
        match self {
            Self::Retained(value) => value.preimage_state_key(original, locus.entity(), &locator),
            Self::Captured(_) => self.retained().map_or(State::Uncaptured, |image| {
                image.original_field_state(original, locus.entity(), &locator)
            }),
        }
    }
    fn family_member_suffix(
        &self,
        family: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        index: usize,
    ) -> Option<&str> {
        self.outputs()
            .native_witness_roles()
            .filter(|(name, posture, entity, _)| {
                name.starts_with(family.prefix())
                    && family.postures().allows(*posture)
                    && *entity == family.entity()
            })
            .nth(index)
            .and_then(|(name, _, _, _)| name.strip_prefix(family.prefix()))
    }
    fn family_member_entity(
        &self,
        family: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        suffix: &str,
    ) -> Option<worth_foundational::facade::EntityId> {
        self.outputs()
            .native_witness_roles()
            .find(|(name, posture, entity, _)| {
                name.strip_prefix(family.prefix()) == Some(suffix)
                    && family.postures().allows(*posture)
                    && *entity == family.entity()
            })
            .map(|(_, _, _, id)| historical_entity(id))
    }
    fn field_for_family_member(
        &self,
        family: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor,
        suffix: &str,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        let original = self
            .outputs()
            .native_witness_roles()
            .find(|(name, posture, entity, _)| {
                name.strip_prefix(family.prefix()) == Some(suffix)
                    && family.postures().allows(*posture)
                    && *entity == family.entity()
            })?
            .3;
        self.retained()?
            .fields()
            .iter()
            .find(|field| {
                field.target_record()
                    == &worth_relational::facade::transactions::RecordRef::Entity(original)
                    && field.locus().entity() == locus.entity()
                    && field.locus().aspect() == locus.aspect()
                    && field.locus().field() == locus.field()
            })
            .map(|field| field.value())
    }
    fn output_entity(
        &self,
        role: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor,
    ) -> Option<worth_foundational::facade::EntityId> {
        let mut roles =
            self.outputs()
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let (_, _, _, id) = roles.next()?;
        if roles.next().is_some() {
            return None;
        }
        Some(historical_entity(id))
    }

    fn relation_count(
        &self,
        locus: &worth_query_declaration::facade::application_aftermath::DeclaredPreImageRelationLocus<S>,
    ) -> Option<usize> {
        let retained = self.retained()?;
        if !retained.relation_loci().iter().any(|value| {
            value.relation() == locus.relation()
                && value.from() == locus.from()
                && value.to() == locus.to()
        }) {
            return None;
        }
        Some(
            retained
                .relations()
                .iter()
                .filter(|row| {
                    row.locus().relation() == locus.relation()
                        && row.locus().from() == locus.from()
                        && row.locus().to() == locus.to()
                })
                .count(),
        )
    }
    fn relation_at(&self, locus: &worth_query_declaration::facade::application_aftermath::DeclaredPreImageRelationLocus<S>, index: usize) -> Option<worth_query_declaration::facade::application_aftermath::ApplicationCorrectionRelationMaterial>{
        let row = self
            .retained()?
            .relations()
            .iter()
            .filter(|row| {
                row.locus().relation() == locus.relation()
                    && row.locus().from() == locus.from()
                    && row.locus().to() == locus.to()
            })
            .nth(index)?;
        let describe = |(from, to)| (historical_entity(from), historical_entity(to));
        Some(worth_query_declaration::facade::application_aftermath::ApplicationCorrectionRelationMaterial::from_historical_endpoints(row.mutation().before().map(describe), row.mutation().after().map(describe)))
    }

    fn field(
        &self,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        match self {
            Self::Retained(value) => {
                value.preimage_value(locus.entity(), locus.aspect(), locus.field())
            }
            Self::Captured(_) => self
                .retained()?
                .fields()
                .iter()
                .find(|field| {
                    field.locus().entity() == locus.entity()
                        && field.locus().aspect() == locus.aspect()
                        && field.locus().field() == locus.field()
                })
                .map(|field| field.value()),
        }
    }
    fn field_for_output(
        &self,
        role: &worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor,
        locus: &DeclaredPreImageLocus<S>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        if role.entity() != locus.entity() {
            return None;
        }
        let mut roles =
            self.outputs()
                .native_witness_roles()
                .filter(|(name, posture, entity, _)| {
                    *name == role.name() && *posture == role.posture() && *entity == role.entity()
                });
        let (_, _, _, original) = roles.next()?;
        if roles.next().is_some() {
            return None;
        }
        if let Self::Retained(value) = self {
            return value.preimage_key(
                original,
                locus.entity(),
                &worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Planned,
                    worth_foundational::facade::AspectKey::new(locus.aspect())?,
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new(locus.field())?,
                    ),
                ),
            );
        }
        self.retained()?
            .fields()
            .iter()
            .find(|field| {
                field.target_record()
                    == &worth_relational::facade::transactions::RecordRef::Entity(original)
                    && field.locus().entity() == locus.entity()
                    && field.locus().aspect() == locus.aspect()
                    && field.locus().field() == locus.field()
            })
            .map(|field| field.value())
    }
}
