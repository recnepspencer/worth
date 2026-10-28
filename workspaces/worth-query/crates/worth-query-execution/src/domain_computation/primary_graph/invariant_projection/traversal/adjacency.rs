//! All projection adjacency callers acquire physical records through this owner.
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::BoundedAdjacencyTruthRead,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn read_adjacency(
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
                self.work_budget.remaining(),
                |_| Ok(()),
            ),
            RelationDirection::Reverse => view.bounded_incoming_relations_of_kind_admitted(
                self.execution,
                member,
                kind,
                self.work_budget.remaining(),
                |_| Ok(()),
            ),
        }
        .map_err(|cause| self.walk_stop(cause, subject))?;
        let read = read.map_err(|limit| self.adjacency_limit_denial(limit, subject))?;
        if !self.work_budget.consume(read.work_units()) {
            return Err(work_budget_exceeded(subject));
        }
        self.work
            .record_adjacencies(1, read.relation_records_examined());
        Ok(read)
    }
}
