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
        self.body_directed_selection(root, selection, Direction::Outgoing, execution, context)
    }

    fn body_directed_selection<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        selection: &ApplicationOperationDecisionReadTarget,
        direction: Direction,
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
            direction,
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
        let terminal = match direction {
            Direction::Outgoing => to,
            Direction::Incoming => from,
        };
        let kind = self.reader.layout.entity_kind(terminal).ok_or_else(|| {
            HandlerExecutionDenial::new(Denial::new(Kind::MissingAuthoritativeFact, terminal))
        })?;
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind,
            entity: terminal.clone().into(),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        let target = self
            .mutation_target(&identity)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(Some(PreparedBodySelection { identity, target }))
    }

    pub(in crate::domain_computation::primary_graph) fn body_path_selection<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        path: &worth_query_declaration::facade::application_query::ApplicationQueryRootPathMeaning,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<Option<PreparedBodySelection<Schema>>, HandlerExecutionDenial> {
        if !path.guards().is_empty() || root.entity.as_ref() != path.start_entity() {
            return Err(HandlerExecutionDenial::new(Denial::new(
                Kind::MissingAuthoritativeFact,
                path.start_entity(),
            )));
        }
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id: root.entity_id,
            kind: root.kind,
            entity: root.entity.clone(),
            authority_identity: root.authority_identity,
            _marker: std::marker::PhantomData,
        };
        let target = self
            .mutation_target(&identity)
            .map_err(HandlerExecutionDenial::new)?;
        let mut current = PreparedBodySelection { identity, target };
        for step in path.steps() {
            if current.identity.entity.as_ref() != step.parent_entity() {
                return Err(HandlerExecutionDenial::new(Denial::new(
                    Kind::MissingAuthoritativeFact,
                    step.parent_entity(),
                )));
            }
            let selection = ApplicationOperationDecisionReadTarget::Relation {
                relation: step.relation().into(),
                from: step.from().into(),
                to: step.to().into(),
            };
            let direction = match step.direction() {
                worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Forward => Direction::Outgoing,
                worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Reverse => Direction::Incoming,
            };
            let Some(selected) = self.body_directed_selection(
                &current.identity,
                &selection,
                direction,
                execution,
                context,
            )?
            else {
                return Ok(None);
            };
            current = selected;
        }
        if current.identity.entity.as_ref() != path.terminal_entity() {
            return Err(HandlerExecutionDenial::new(Denial::new(
                Kind::MissingAuthoritativeFact,
                path.terminal_entity(),
            )));
        }
        Ok(Some(current))
    }
}
