//! One decision-owned selection supplies related field reads and predecessor retirement.
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantEntityIdentity;
use crate::domain_computation::request_execution::QueryMemoryReservation;
use worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget;

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
