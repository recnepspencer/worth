//! Selected projection adjacency guards every ordinary and index DATA reader.
use super::ring_source::RingFixture;
use std::cell::Cell;
use worth_relational::facade::{
    identity::KindId, transactions::RelationalExecutionDenialCause as Execution,
};
#[derive(Clone, Copy, Debug)]
pub(super) enum Door {
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
    pub(super) const ALL: [Self; 9] = [
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
    pub(super) fn incoming(self) -> bool {
        matches!(
            self,
            Self::BoundedIncoming
                | Self::AdmittedIncoming
                | Self::FrontierIncoming
                | Self::IndexIncoming
        )
    }
    pub(super) fn read(
        self,
        cold: &RingFixture,
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
                .try_project_historical_version(super::current_read_basis::main_selected_version(
                    &cold.runtime,
                ))
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
                    if let Some(symbol)=cold.remote_symbol {
                        assert!(matches!(super::relation_field_values::label(&records[0]),Some(worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Symbol(actual))) if actual.0==symbol.0));
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
                .map(|answer| answer.unwrap().into_records().len()),
            Self::FrontierIncoming => view
                .bounded_incoming_relations_for_frontier(execution, &frontier, KindId(2), Some(32))
                .map(|answer| answer.unwrap().into_records().len()),
            Self::IndexOutgoing => view
                .bounded_index_relations_for_frontier(
                    execution,
                    &frontier,
                    KindId(2),
                    true,
                    Some(32),
                )
                .map(|answer| answer.unwrap().into_records().len()),
            Self::IndexIncoming => view
                .bounded_index_relations_for_frontier(
                    execution,
                    &frontier,
                    KindId(2),
                    false,
                    Some(32),
                )
                .map(|answer| answer.unwrap().into_records().len()),
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
