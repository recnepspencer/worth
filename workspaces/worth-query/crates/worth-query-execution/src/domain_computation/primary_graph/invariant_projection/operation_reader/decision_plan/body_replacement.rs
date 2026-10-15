//! The bounded adjacency observer seals optional predecessor selection and liveness.
use super::{
    ApplicationSchema, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    application_contribution::QueryRequestExecution, HandlerExecutionDenial,
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryInvariantMutationTarget,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLink,
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Source,
};
impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn body_creation_predecessor<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        link: &ApplicationOperationBodyCreationLink,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<WorthQueryInvariantMutationTarget<Schema, ()>>, HandlerExecutionDenial> {
        let source = Source::RelatedItemCount {
            relation: link.relation.clone(),
            from: link.from.clone(),
            to: link.to.clone(),
            direction: Direction::Outgoing,
        };
        let selected = self.observed_requirement_collection(root, &source, execution, context)?;
        let entity_id = match selected.ids.as_slice() {
            [] => return Ok(None),
            [id] => *id,
            _ => {
                return Err(HandlerExecutionDenial::new(Denial::new(
                    Kind::AmbiguousRelation,
                    &link.relation,
                )))
            }
        };
        let kind = self.reader.layout.entity_kind(&link.to).ok_or_else(|| {
            HandlerExecutionDenial::new(Denial::new(Kind::MissingAuthoritativeFact, &link.to))
        })?;
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind,
            entity: link.to.clone().into(),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        self.mutation_target(&identity)
            .map(Some)
            .map_err(HandlerExecutionDenial::new)
    }
}
