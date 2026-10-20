//! Every certified profile rejects candidate writes to its actual observed Subject loci.
use super::{identity, BodyCandidate, Denial};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationRealizedEffect as Effect;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};
use worth_relational::facade::identity::EntityId;

/// O(candidate effects), no allocation or graph reread; called only for Subject captures.
/// Actual identity catches distinct paths that alias the same entity and field.
pub(super) fn require_unchanged<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    candidate: &BodyCandidate<S, B>,
    source: EntityId,
    field: &AspectFieldLocator,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    for effect in &candidate.effects {
        identity::check_interruption(context)?;
        let same_field = |candidate: &AspectFieldLocator| {
            candidate.aspect().aspect_key() == field.aspect().aspect_key()
                && candidate.field_path() == field.field_path()
        };
        let touched = match effect {
            Effect::UpdateEntity {
                entity_id, fields, ..
            } => *entity_id == source && fields.keys().any(same_field),
            Effect::PatchOptionalEntityFields {
                entity_id, fields, ..
            } => *entity_id == source && fields.keys().any(same_field),
            Effect::DeleteEntity { entity_id } => *entity_id == source,
            _ => false,
        };
        if touched {
            return Err(Denial::CandidateMismatch);
        }
    }
    Ok(())
}
