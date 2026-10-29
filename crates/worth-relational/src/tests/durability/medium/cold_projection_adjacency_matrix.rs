//! Actual selected-root adjacency doors must guard before yielding values or callbacks.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use std::cell::Cell;
#[derive(Clone, Copy, Debug)]
enum Door {
    Outgoing,
    BoundedOutgoing,
    BoundedIncoming,
    AdmittedOutgoing,
    AdmittedIncoming,
    FrontierOutgoing,
    FrontierIncoming,
    IndexOutgoing,
    IndexIncoming,
}
impl Door {
    const ALL: [Self; 9] = [
        Self::Outgoing,
        Self::BoundedOutgoing,
        Self::BoundedIncoming,
        Self::AdmittedOutgoing,
        Self::AdmittedIncoming,
        Self::FrontierOutgoing,
        Self::FrontierIncoming,
        Self::IndexOutgoing,
        Self::IndexIncoming,
    ];
    fn incoming(self) -> bool {
        matches!(
            self,
            Self::BoundedIncoming
                | Self::AdmittedIncoming
                | Self::FrontierIncoming
                | Self::IndexIncoming
        )
    }
    fn read(
        self,
        cold: &ColdRoot,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        historical: bool,
    ) -> Result<(), Execution> {
        let (_, basis) = cold
            .runtime
            .observe_branch(&cold.runtime.main_branch_identity())
            .unwrap();
        let view = if historical {
            cold.runtime
                .read_truth()
                .try_project_historical_version(cold.runtime.current_version_id())
                .unwrap()
        } else {
            cold.runtime
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap()
        };
        let anchor = cold.entities[0];
        let callbacks = Cell::new(0);
        let mut admit = |_| {
            callbacks.set(callbacks.get() + 1);
            Ok(())
        };
        let frontier = std::collections::BTreeSet::from([anchor]);
        let answer = match self {
            Self::Outgoing => view
                .outgoing_relations_of_kind(execution, anchor, KindId(2))
                .map(|records| records.len()),
            Self::BoundedOutgoing => view
                .bounded_outgoing_relations_of_kind(execution, anchor, KindId(2), Some(32))
                .map(|answer| answer.unwrap().into_records().len()),
            Self::BoundedIncoming => view
                .bounded_incoming_relations_of_kind(execution, anchor, KindId(2), Some(32))
                .map(|answer| {
                    let records = answer.unwrap().into_records();
                    if let Some(symbol) = cold.remote_symbol {
                        let value = crate::visibility::materialization::read_records::relation_query_locus_value(&records[0], &aspect_field_locator(aspect_key("label"), field_key("label")));
                        assert_eq!(cold.runtime.services.symbols.interner_snapshot().resolve(symbol), Some("remote-paged-edge"), "actual incoming value={value:?}; member reads={:?}", cold.source.member_reads.lock().unwrap());
                    }
                    records.len()
                }),
            Self::AdmittedOutgoing => view
                .bounded_outgoing_relations_of_kind_admitted(
                    execution,
                    anchor,
                    KindId(2),
                    Some(32),
                    &mut admit,
                )
                .map(|answer| answer.unwrap().into_records().len()),
            Self::AdmittedIncoming => view
                .bounded_incoming_relations_of_kind_admitted(
                    execution,
                    anchor,
                    KindId(2),
                    Some(32),
                    &mut admit,
                )
                .map(|answer| answer.unwrap().into_records().len()),
            Self::FrontierOutgoing => view
                .bounded_outgoing_relations_for_frontier(execution, &frontier, KindId(2), Some(32))
                .map(|answer| answer.unwrap().records.len()),
            Self::FrontierIncoming => view
                .bounded_incoming_relations_for_frontier(execution, &frontier, KindId(2), Some(32))
                .map(|answer| answer.unwrap().records.len()),
            Self::IndexOutgoing => view
                .bounded_index_relations_for_frontier(
                    execution,
                    &frontier,
                    KindId(2),
                    true,
                    Some(32),
                )
                .map(|answer| answer.unwrap().records.len()),
            Self::IndexIncoming => view
                .bounded_index_relations_for_frontier(
                    execution,
                    &frontier,
                    KindId(2),
                    false,
                    Some(32),
                )
                .map(|answer| answer.unwrap().records.len()),
        };
        if answer.is_err() {
            assert_eq!(
                callbacks.get(),
                0,
                "{self:?} exposes no provisional callback"
            );
        }
        answer.map(|count| assert_eq!(count, 1, "the actual ring has one selected edge: {self:?}"))
    }
}
#[derive(Clone, Copy, Debug)]
enum Subject {
    AnchorHead,
    AnchorHistory,
    RemoteHead,
    RemoteHistory,
    AnchorSymbols,
    RemoteSymbols,
}
impl Subject {
    const ALL: [Self; 6] = [
        Self::AnchorHead,
        Self::AnchorHistory,
        Self::RemoteHead,
        Self::RemoteHistory,
        Self::AnchorSymbols,
        Self::RemoteSymbols,
    ];
    fn remote(self) -> bool {
        matches!(
            self,
            Self::RemoteHead | Self::RemoteHistory | Self::RemoteSymbols
        )
    }
    fn symbols(self) -> bool {
        matches!(self, Self::AnchorSymbols | Self::RemoteSymbols)
    }
    fn history(self) -> bool {
        matches!(self, Self::AnchorHistory | Self::RemoteHistory)
    }
    fn required(self, door: Door, historical: bool) -> bool {
        (!self.remote() || door.incoming()) && (!self.history() || historical)
    }
}

#[test]
fn projection_adjacency_discovery_settles_only_one_accepted_round() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in Door::ALL {
        for historical in [false, true] {
            let cold = recover_shape(execution, None, 4, true);
            cold.runtime.performance_access().reset_counters();
            door.read(&cold, execution, historical).unwrap();
            let initial = cold.runtime.performance_access().counters();
            assert!(cold.source.reads.load(Ordering::Relaxed) > 0);
            cold.runtime.performance_access().reset_counters();
            door.read(&cold, execution, historical).unwrap();
            let warm = cold.runtime.performance_access().counters();
            assert!(warm.adjacency_kind_slices_leased > 0);
            assert_eq!(
                initial.adjacency_kind_slices_leased, warm.adjacency_kind_slices_leased,
                "{door:?} historical={historical}"
            );
            assert_eq!(
                initial.visible_authoritative_relation_records_materialized,
                warm.visible_authoritative_relation_records_materialized
            );
        }
    }
}
#[test]
fn an_exhausted_frontier_never_demands_an_unvisited_anchor() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for historical in [false, true] {
        let cold = recover_shape(execution, None, 4, true);
        cold.source.damaged.store(true, Ordering::Relaxed);
        let (_, basis) = cold
            .runtime
            .observe_branch(&cold.runtime.main_branch_identity())
            .unwrap();
        let view = if historical {
            cold.runtime
                .read_truth()
                .try_project_historical_version(cold.runtime.current_version_id())
                .unwrap()
        } else {
            cold.runtime
                .read_truth()
                .project_observation(&basis.observation())
                .unwrap()
        };
        assert!(view
            .bounded_incoming_relations_for_frontier(
                execution,
                &std::collections::BTreeSet::from([cold.entities[0]]),
                KindId(2),
                Some(0)
            )
            .unwrap()
            .is_err());
        assert_eq!(cold.source.reads.load(Ordering::Relaxed), 0);
    }
}

#[cfg(test)]
#[path = "cold_projection_adjacency_matrix/exhaustive.rs"]
mod exhaustive;
