//! One decision-owned selection supplies related field reads and predecessor retirement.
use super::WorthQueryInvariantMutationTarget;
use crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;
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

pub(super) fn installed_reads(
    body: &worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
) -> Vec<ApplicationOperationDecisionReadTarget> {
    body.creation()
        .into_iter()
        .flat_map(|root| root.initialization_sites())
        .flat_map(|creation| {
            creation
                .fields()
                .iter()
                .filter_map(|step| step.related_source().map(|source| source.selection_read()))
                .chain(
                    creation
                        .links()
                        .iter()
                        .flat_map(|link| link.replacement_reads())
                        .filter(|read| {
                            matches!(
                                read,
                                ApplicationOperationDecisionReadTarget::Relation { .. }
                            )
                        }),
                )
        })
        .chain(
            body.outputs()
                .iter()
                .filter_map(|output| output.target.selection_read()),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
