//! Borrow scalar projections from the actual Native-admitted observation.
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
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
    pub(crate) relations: &'source StructuralRelationView<'source>,
    pub(crate) aspects: &'source StructuralAspectStateView<'source>,
    pub(crate) audience: AdmittedManagedIndexSourceAudience,
}

impl<'source> ManagedIndexSourceProjection<'source> {
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        self.program
    }

    /// Full generation is checked by the actual Native view before any field
    /// access. Missing entities are not an empty projection.
    pub(crate) fn field(
        &self,
        row: EntityId,
        column: usize,
    ) -> Result<Option<&AspectValue>, Denial> {
        if !self.program.0.authority.admits(&self.audience) {
            return Err(Definition::OutsideDeclaredAccess.into());
        }
        self.relations
            .read_budget()
            .checkpoint(1)
            .map_err(Denial::Structural)?;
        if self
            .relations
            .entity_kind(row)
            .map_err(|cause| Denial::Structural(self.relations.read_budget().refusal(cause)))?
            != self.program.row_kind()
        {
            return Err(Definition::InvalidMemberDomain.into());
        }
        let field = self
            .program
            .fields()
            .get(column)
            .ok_or(Definition::InvalidExpression)?;
        let state = self
            .aspects
            .entity_aspect_state(row)
            .map_err(|cause| Denial::Structural(self.relations.read_budget().refusal(cause)))?;
        let Some(aspect) = state.get(field.locator.aspect().aspect_key()) else {
            return Ok(None);
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
        Ok(fields.get(key))
    }
}
