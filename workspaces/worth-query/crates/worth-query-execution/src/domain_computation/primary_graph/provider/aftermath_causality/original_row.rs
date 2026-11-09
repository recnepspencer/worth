//! Original causal meaning is bounded by the exact Native creation delta.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn original_aftermath_causality(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        publication: &super::super::OriginalPublication,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
        outcome: crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcomeIdentity,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        use WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships;
        execution
            .consult()
            .map_err(|cause| aftermath_read_stop(cause.into()))?;
        let layout = self.graph.layout.provider_aftermath_causality();
        let semantic = publication.semantic_result();
        let outcome_field = layout
            .outcome_identity_locator
            .field_path()
            .fields()
            .first()
            .ok_or(InconsistentCanonicalRelationships)?;
        let mut selected = None;
        for created in semantic
            .created_entities()
            .iter()
            .filter(|created| created.key.kind == layout.entity_kind)
        {
            let row = publication
                .created_entity(execution, created.assigned)
                .map_err(|cause| {
                    if let worth_relational::facade::durability::RelationalAssociatedEffectRefusal::Recovery(
                        worth_relational::facade::durability::RelationalRecoveryStop::Request(cause)
                    ) = cause { aftermath_read_stop(cause) } else { InconsistentCanonicalRelationships }
                })?
                .ok_or(InconsistentCanonicalRelationships)?;
            let Some(AspectValue::UInt64(recorded_outcome)) = row.field(
                layout.outcome_identity_locator.aspect().aspect_key(),
                outcome_field,
            ) else {
                return Err(InconsistentCanonicalRelationships);
            };
            if *recorded_outcome != outcome.get() {
                continue;
            }
            if selected.replace((created.assigned, row)).is_some() {
                return Err(InconsistentCanonicalRelationships);
            }
        }
        let Some((entity, row)) = selected else {
            return Ok(None);
        };
        let fields = required_fields(layout)?;
        let aspect = layout.key_locator.aspect().aspect_key();
        let values = fields
            .iter()
            .map(|field| row.field(aspect, field).cloned())
            .collect();
        let fact = canonical_record::decode(
            runtime,
            entity,
            (semantic.receipt().version_id, values),
            owner,
        )?;
        if &fact.child != semantic.receipt() {
            return Err(InconsistentCanonicalRelationships);
        }
        Ok(Some(fact))
    }
}
