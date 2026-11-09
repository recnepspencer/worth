//! Borrow scalar projections from the actual Native-admitted observation.
use crate::runtime::RelationalProjectedFieldObservation as Observation;
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_proof::AdmittedManagedIndexSourceAudience;

use super::program::ManagedIndexSourceProgram;
use crate::derived_relation::{
    DerivedRelationDenial as Definition, DerivedRelationReadDenial as Denial,
};
use crate::identity::data::EntityId;
use crate::validation::data::{StructuralAspectStateView, StructuralRelationView};

/// Valid only while its actual preparation/observation borrow is alive. The
/// concrete audience witness is consumed with Native views, never in isolation.
pub struct ManagedIndexSourceProjection<'source> {
    pub(crate) program: &'source ManagedIndexSourceProgram,
    pub(crate) backing: SourceProjectionBacking<'source>,
}

impl<'source> ManagedIndexSourceProjection<'source> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.program
    }

    pub(super) fn field_at(
        &self,
        row: EntityId,
        kind: crate::identity::data::KindId,
        field: &super::ManagedIndexSourceField,
    ) -> Result<Observation<'_>, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant {
                relations,
                aspects,
                audience,
                ..
            } => {
                if !self.program.0.authority.admits(audience) {
                    return Err(Definition::OutsideDeclaredAccess.into());
                }
                self.checkpoint(1)?;
                if relations
                    .entity_kind(row)
                    .map_err(|cause| Denial::Structural(relations.read_budget().refusal(cause)))?
                    != kind
                {
                    return Err(Definition::InvalidMemberDomain.into());
                }
                let state = aspects
                    .entity_aspect_state(row)
                    .map_err(|cause| Denial::Structural(relations.read_budget().refusal(cause)))?;
                field_value(state.get(field.locator.aspect().aspect_key()), field)
            }
            SourceProjectionBacking::Selected { .. } => Err(Definition::ForeignObservation.into()),
        }
    }
}

fn field_value<'a>(
    aspect: Option<&'a worth_foundational::facade::ContractValidatedAspectValue>,
    field: &super::ManagedIndexSourceField,
) -> Result<Observation<'a>, Denial> {
    let Some(aspect) = aspect else {
        return Ok(Observation::AspectAbsent);
    };
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return Err(Definition::InvalidExpression.into());
    };
    let key = field
        .locator
        .field_path()
        .fields()
        .first()
        .ok_or(Definition::InvalidExpression)?;
    Ok(fields
        .get(key)
        .map_or(Observation::FieldAbsent, Observation::Present))
}
mod access;
mod content;
pub use access::PreparedManagedIndexRelationRead;
pub(crate) use access::SourceProjectionBacking;
pub(in crate::validation::candidate_source) use access::SourceProjectionRetention;
