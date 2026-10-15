//! The bounded adjacency observer seals optional predecessor selection and liveness.
use super::{
    ApplicationSchema, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    application_contribution::QueryRequestExecution, operation_body::PreparedBodySelection,
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Source, ApplicationOperationDecisionReadTarget,
};
impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn body_related_selection<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        selection: &ApplicationOperationDecisionReadTarget,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<PreparedBodySelection<Schema>>, HandlerExecutionDenial> {
        let ApplicationOperationDecisionReadTarget::Relation { relation, from, to } = selection
        else {
            unreachable!("installed body relation selection");
        };
        let source = Source::RelatedItemCount {
            relation: relation.clone(),
            from: from.clone(),
            to: to.clone(),
            direction: Direction::Outgoing,
        };
        let selected = self.observed_requirement_collection(root, &source, execution, context)?;
        let entity_id = match selected.ids.as_slice() {
            [] => return Ok(None),
            [id] => *id,
            _ => {
                return Err(HandlerExecutionDenial::new(Denial::new(
                    Kind::AmbiguousRelation,
                    relation,
                )))
            }
        };
        let kind = self.reader.layout.entity_kind(to).ok_or_else(|| {
            HandlerExecutionDenial::new(Denial::new(Kind::MissingAuthoritativeFact, to))
        })?;
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind,
            entity: to.clone().into(),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        let target = self
            .mutation_target(&identity)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(Some(PreparedBodySelection { identity, target }))
    }
}
