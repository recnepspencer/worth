//! Concrete native adjacency acquisition and endpoint validation.
use super::super::traversal::{endpoint_unavailable, foreign, work_budget_exceeded};
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    identity::RelationId,
    runtime::{BoundedAdjacencyTruthRead, ProjectionAspectScope, VisibilityProjectionView},
    storage::RecordLifecycleState,
};
impl NativeRead<'_, '_> {
    pub fn relations(
        &mut self,
        name: &str,
        source: &str,
        destination: &str,
        anchor: NativeIdentity<'_>,
        direction: RelationDirection,
        receive: &mut dyn FnMut(
            RelationId,
            WorthQueryInvariantEntityIdentity<(), ()>,
            WorthQueryInvariantEntityIdentity<(), ()>,
        ),
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        let layout = self.layout.relation(name).cloned().ok_or_else(|| {
            WorthQueryInvariantProjectionTraversalDenial::new(
                WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled,
                name,
            )
        })?;
        let (entity, kind) = match direction {
            RelationDirection::Forward => (source, layout.from),
            RelationDirection::Reverse => (destination, layout.to),
        };
        if !self.local(anchor, entity) || anchor.kind != kind {
            return Err(foreign(name));
        }
        let read = self.read_adjacency(name, anchor.entity_id, layout.kind, direction)?;
        for record in read.into_records() {
            self.scope.record_relation(record.source, record.target);
            let (from, to) = match direction {
                RelationDirection::Forward => (
                    anchor.owned(),
                    self.endpoint(self.execution, record.target, layout.to, destination)?,
                ),
                RelationDirection::Reverse => (
                    self.endpoint(self.execution, record.source, layout.from, source)?,
                    anchor.owned(),
                ),
            };
            receive(record.relation_id, from, to);
        }
        Ok(())
    }
    pub fn endpoint(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        expected_kind: KindId,
        entity: &str,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<(), ()>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let view = self.branch_view(entity)?;
        let work = &mut self.work;
        let available = view
            .entity_record_with_projection_scope_admitted(
                execution,
                entity_id,
                ProjectionAspectScope::empty(),
                |_| Ok::<_, std::convert::Infallible>(()),
                |record| {
                    work.record_endpoint();
                    (record.kind_id() == expected_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then_some(())
                },
            )
            .map(|result| result.unwrap_or_else(|never| match never {}))
            .map_err(|cause| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(
                        crate::facade::primary_graph::relational_execution_kind(cause, None),
                    ),
                    entity,
                )
            })?
            .is_some();
        let available = if !available && self.native.candidate_identity().is_some() {
            // An authored prefix can retire an endpoint before removing its edge.
            // Exact retirement metadata grants no payload or equality access.
            if !self.budget.consume(1) {
                return Err(work_budget_exceeded(entity));
            }
            self.work.record_endpoint();
            view.read_entity_retirement(execution, entity_id)
                .map_err(|cause| self.walk_stop(cause, entity))?
                .is_some_and(|retired| retired.kind_id() == expected_kind)
        } else {
            available
        };
        if !available {
            return Err(endpoint_unavailable(entity));
        }
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: expected_kind,
            entity: Arc::from(entity),
            authority_identity: self.authority,
            phase: self
                .native
                .candidate_identity()
                .map(|id| id.qualification()),
            _marker: PhantomData,
        })
    }
    pub fn branch_view(
        &self,
        subject: &str,
    ) -> Result<VisibilityProjectionView, WorthQueryInvariantProjectionTraversalDenial> {
        self.native
            .projection(self.snapshot)
            .ok_or_else(|| endpoint_unavailable(subject))
    }
    pub fn admit_adjacency_list(
        &mut self,
        relation: &str,
    ) -> Result<(), WorthQueryInvariantProjectionTraversalDenial> {
        if !self.budget.can_afford(1) {
            return Err(work_budget_exceeded(relation));
        }
        assert!(self.budget.consume(1), "projection work was admitted");
        Ok(())
    }
    pub fn adjacency_limit_denial(
        &mut self,
        limit: worth_relational::facade::runtime::AdjacencyTruthReadLimitExceeded,
        relation: &str,
    ) -> WorthQueryInvariantProjectionTraversalDenial {
        if self.budget.consume(limit.consumed_work_units()) {
            self.work
                .record_adjacencies(1, limit.relation_records_examined());
        }
        self.budget.mark_exceeded();
        work_budget_exceeded(relation)
    }
    pub fn read_adjacency(
        &mut self,
        subject: &str,
        member: EntityId,
        kind: KindId,
        direction: RelationDirection,
    ) -> Result<BoundedAdjacencyTruthRead, WorthQueryInvariantProjectionTraversalDenial> {
        self.admit_adjacency_list(subject)?;
        let view = self.branch_view(subject)?;
        let read = match direction {
            RelationDirection::Forward => view.bounded_outgoing_relations_of_kind_admitted(
                self.execution,
                member,
                kind,
                self.budget.remaining(),
                |_| Ok(()),
            ),
            RelationDirection::Reverse => view.bounded_incoming_relations_of_kind_admitted(
                self.execution,
                member,
                kind,
                self.budget.remaining(),
                |_| Ok(()),
            ),
        }
        .map_err(|cause| self.walk_stop(cause, subject))?;
        let read = read.map_err(|limit| self.adjacency_limit_denial(limit, subject))?;
        if !self.budget.consume(read.work_units()) {
            return Err(work_budget_exceeded(subject));
        }
        self.work
            .record_adjacencies(1, read.relation_records_examined());
        Ok(read)
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn walk_stop(
        &mut self,
        cause: RelationalExecutionDenialCause,
        subject: &str,
    ) -> WorthQueryInvariantProjectionTraversalDenial {
        self.stopped.get_or_insert(cause);
        WorthQueryInvariantProjectionTraversalDenial::new(
            WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(
                crate::facade::primary_graph::relational_execution_kind(cause, None),
            ),
            subject,
        )
    }
}
