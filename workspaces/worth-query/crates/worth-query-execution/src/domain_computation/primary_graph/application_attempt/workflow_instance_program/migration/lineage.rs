//! The one successor a migrated source links from.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::super::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

/// The identity of the one successor a migrated source links from.
pub(super) fn successor_identity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    source: EntityId,
) -> Result<Option<String>, super::super::super::WorthQueryApplicationAttemptDenial> {
    let successors = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.instance_migrated_from_relation,
        source,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
    )?;
    let Some(successors) = successors else {
        return Ok(None);
    };
    let [successor] = successors.as_slice() else {
        return Ok(None);
    };
    Ok(
        match observe_field_value(
            runtime,
            execution,
            snapshot,
            successor.from,
            layout.instance.entity_kind,
            &layout.instance.identity,
        )? {
            Some(AspectValue::String(InternedString::Raw(identity))) => Some(identity),
            _ => None,
        },
    )
}

#[cfg(test)]
mod request_stops {
    use super::*;
    use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
    #[test]
    fn successor_read_keeps_cancellation_at_the_native_frontier() {
        let _observation =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        let world = installed_authorization_world(true);
        world.application.with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let graph = world.application.runtime.primary_graph().unwrap();
            let request = worth_execution::CancellationSource::new();
            let serial = worth_execution::SerialRequest::from_memory(worth_execution::SerialMemoryBudget::new(None), request.token(), None);
            let cancelled = worth_execution::ExecutionRequest::serial(&serial);
            let anchor = selected.resolve_entity(&phase, crate::domain_computation::primary_graph::tests::fixture::AccountStatus::reference(), "open".to_owned(), &crate::domain_computation::primary_graph::tests::fixture::live_scope(), crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id();
            graph.integration_handle().with_runtime(&execution, |runtime| {
                let snapshot = selected.application_basis().snapshot_handle();
                let _probe = worth_execution::ExecutionCheckpointObservationForTest::install_on_this_thread_for_test(move || request.cancel());
                let denial = successor_identity(&cancelled, runtime, snapshot, graph.layout.workflow(), anchor).unwrap_err();
                assert_eq!(denial.kind(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::Cancelled);
            });
        }).unwrap();
    }
}
