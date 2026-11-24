//! Borrow scalar projections from the actual Native-admitted observation.
use crate::contracts::visibility::materialization::read_records::projection::projection_records::field_observation::RelationalProjectedFieldObservation as Observation;
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_proof::AdmittedManagedIndexSourceAudience;
use crate::contracts::validation::candidate_source::program::{ManagedIndexSourceProgram, ManagedIndexSourceProgramFieldsPort as _};

use crate::contracts::derived_relation::definition::DerivedRelationDenial as Definition;
use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::validation::custom_rule::structural_views::StructuralAspectStateView;
use crate::validation::custom_rule::structural_views::StructuralRelationView;

/// Valid only while its actual preparation/observation borrow is alive. The
/// concrete audience witness is consumed with Native views, never in isolation.
pub struct ManagedIndexSourceProjection<'source> {
    pub(crate) program: &'source ManagedIndexSourceProgram,
    pub(in crate::internal_port::validation::candidate_source) backing:
        SourceProjectionBacking<'source>,
}

impl<'source> ManagedIndexSourceProjection<'source> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.program
    }

    pub(super) fn field_at(
        &self,
        row: EntityId,
        kind: crate::contracts::identity::data::primitives::KindId,
        field: &super::ManagedIndexSourceField,
    ) -> Result<Observation<'_>, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant {
                relations,
                aspects,
                audience,
                ..
            } => {
                if !self.program.owner_field_0().authority.admits(audience) {
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
            SourceProjectionBacking::Selected { .. } | SourceProjectionBacking::Applied { .. } => {
                Err(Definition::ForeignObservation.into())
            }
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
pub mod access;
pub use access::applied_rows;
pub use access::selected_rows;
pub mod content;
pub use access::PreparedManagedIndexRelationRead;
pub(in crate::internal_port::validation::candidate_source) use access::SourceProjectionBacking;
pub(in crate::internal_port::validation::candidate_source) use access::SourceProjectionRetention;
