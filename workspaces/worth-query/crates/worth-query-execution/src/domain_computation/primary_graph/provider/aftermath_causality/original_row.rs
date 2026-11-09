//! Original causal meaning is bounded by the exact Native creation delta.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn original_aftermath_causality(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        publication: &super::super::OriginalPublication,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        use WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships;
        execution
            .consult()
            .map_err(|cause| aftermath_read_stop(cause.into()))?;
        let layout = self.graph.layout.provider_aftermath_causality();
        let semantic = publication.semantic_result();
        let mut rows = semantic
            .created_entities()
            .iter()
            .filter(|created| created.key.kind == layout.entity_kind);
        let Some(created) = rows.next() else {
            return Ok(None);
        };
        if rows.next().is_some() {
            return Err(InconsistentCanonicalRelationships);
        }
        let row = publication
            .created_entity(execution, created.assigned)
            .map_err(|cause| {
                if let worth_relational::facade::durability::RelationalAssociatedEffectRefusal::Recovery(
                    worth_relational::facade::durability::RelationalRecoveryStop::Request(cause)
                ) = cause { aftermath_read_stop(cause) } else { InconsistentCanonicalRelationships }
            })?
            .ok_or(InconsistentCanonicalRelationships)?;
        decode_created(runtime, layout, semantic, &row, owner).map(Some)
    }
}

fn decode_created(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    layout: &WorthQueryAftermathCausalityLayout,
    semantic: &worth_relational::facade::transactions::RelationalCommitSemanticResult,
    row: &worth_relational::facade::durability::ReadmittedRelationalCreatedEntity<'_>,
    owner: worth_foundational::facade::WorldOwnerCoordinate,
) -> Result<WorthQueryCanonicalCorrectionFact, WorthQueryAftermathCausalityReadDenial> {
    use WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships;
    let fields = required_fields(layout)?;
    let aspect = layout.key_locator.aspect().aspect_key();
    let values = fields
        .iter()
        .map(|field| row.field(aspect, field).cloned())
        .collect();
    let fact = canonical_record::decode(
        runtime,
        row.entity_id(),
        (semantic.receipt().version_id, values),
        owner,
    )?;
    if &fact.child != semantic.receipt() {
        return Err(InconsistentCanonicalRelationships);
    }
    Ok(fact)
}
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn readmitted_original_causality(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        effect: &worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        use WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships as Denied;
        let layout = self.graph.layout.provider_aftermath_causality();
        let semantic = effect.semantic_result();
        let mut matching = semantic
            .created_entities()
            .iter()
            .filter(|created| created.key.kind == layout.entity_kind);
        let Some(created) = matching.next() else {
            return Ok(None);
        };
        if matching.next().is_some() {
            return Err(Denied);
        }
        let row = effect
            .created_entity(execution, created.assigned)
            .map_err(|_| Denied)?
            .ok_or(Denied)?;
        decode_created(runtime, layout, semantic, &row, owner).map(Some)
    }
}
