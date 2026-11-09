//! Field-bound bytes retain the exact logical observation and original payer.
use super::*;
use crate::content::RelationalContentFrame;
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial;
use worth_foundational::facade::{AspectFieldLocator, ContractValidatedAspectValueView};

impl StructuralRelationView<'_> {
    /// Reads only this entity's installed content field. The callback borrows
    /// the original request; no new request or Source audience is created.
    pub fn consume_entity_field_content<R>(
        &self,
        entity: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        consume: impl FnOnce(
            worth_execution::ExecutionRequest<'_, '_>,
            Option<RelationalContentFrame>,
        ) -> R,
    ) -> Result<R, Denial> {
        let actual = self.entity_kind(entity).map_err(|cause| {
            Denial::Source(
                crate::derived_relation::DerivedRelationReadDenial::Structural(
                    self.read_budget().refusal(cause),
                ),
            )
        })?;
        if actual != kind {
            return Err(Denial::ForeignSource);
        }
        let (_, source) = self
            .structural_source
            .as_ref()
            .ok_or(Denial::ForeignSource)?;
        let owner = source
            .proposal
            .as_ref()
            .and_then(|proposal| proposal.source_interpretation())
            .ok_or(Denial::ForeignSource)?;
        let expected = if source.committed {
            owner.physical_basis().observation().version_id()
        } else {
            owner
                .suspension_source_version()
                .unwrap_or(owner.proposed_version())
        };
        if source.version != expected {
            return Err(Denial::ForeignSource);
        }
        self.read_budget().checkpoint(1).map_err(|cause| {
            Denial::Source(crate::derived_relation::DerivedRelationReadDenial::Structural(cause))
        })?;
        let state = self
            .state_view
            .entity_aspect_state(entity)
            .ok_or(Denial::ForeignSource)?;
        let [key] = locator.field_path().fields() else {
            return Err(Denial::ForeignSource);
        };
        let value = match state
            .get(locator.aspect().aspect_key())
            .map(|aspect| aspect.view())
        {
            Some(ContractValidatedAspectValueView::Struct(fields)) => fields.get(key),
            None => None,
            _ => return Err(Denial::ForeignSource),
        };
        let frame = owner.content_view(source.committed)?.read_source(
            &self.read_budget(),
            entity,
            kind,
            locator,
            value,
        )?;
        Ok(consume(self.execution, frame))
    }
}
