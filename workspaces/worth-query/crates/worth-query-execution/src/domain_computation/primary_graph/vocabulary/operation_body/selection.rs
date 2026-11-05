//! Moved whole from `domain_computation::primary_graph::operation_body::selection`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use crate::domain_computation::request_execution::QueryMemoryReservation;
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

pub(in crate::domain_computation::primary_graph) struct PreparedBodySelection<Schema> {
    pub(in crate::domain_computation::primary_graph) identity:
        WorthQueryInvariantEntityIdentity<Schema, ()>,
    pub(in crate::domain_computation::primary_graph) target:
        WorthQueryInvariantMutationTarget<Schema, ()>,
}

pub(in crate::domain_computation::primary_graph) struct PreparedBodySelections<Schema> {
    pub(in crate::domain_computation::primary_graph) targets: Vec<(
        ApplicationOperationDecisionReadTarget,
        Option<PreparedBodySelection<Schema>>,
    )>,
    pub(in crate::domain_computation::primary_graph) _memory: Option<QueryMemoryReservation>,
}

impl<Schema> PreparedBodySelections<Schema> {
    pub(in crate::domain_computation::primary_graph) fn selected(
        &self,
        relation: &str,
        from: &str,
        to: &str,
    ) -> &Option<PreparedBodySelection<Schema>> {
        let index = self
            .targets
            .binary_search_by(|(read, _)| {
                let ApplicationOperationDecisionReadTarget::Relation {
                    relation: name,
                    from: source,
                    to: target,
                } = read
                else {
                    unreachable!("installed selection is a relation");
                };
                (name.as_str(), source.as_str(), target.as_str()).cmp(&(relation, from, to))
            })
            .expect("installed selection");
        &self.targets[index].1
    }
}
