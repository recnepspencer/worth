//! Independent assertions against published private rows, memberships and events.
use super::fixtures::*;
use super::read_handoff_rule::Control;
use crate::aggregate_identity::AggregateRead;
use crate::identity::data::EntityId;
use crate::tests::support::*;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{
    AggregateIdentity, AggregateLineageKind, AspectValue, ContractValidatedAspectValueView,
};

pub(super) fn assert_published(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    result: &CommitResult,
    control: &Control,
    before: Option<&crate::snapshots::data::SnapshotHandle>,
) {
    let rows = records(execution, runtime, &result.snapshot);
    let read = runtime
        .read_truth()
        .read_snapshot(&result.snapshot, execution)
        .unwrap()
        .unwrap();
    let row_identity = |entity| {
        rows.iter()
            .find(|(id, _)| *id == entity)
            .unwrap()
            .1
            .identity
    };
    let bindings = read
        .relations()
        .into_iter()
        .filter(|edge| edge.kind.kind_id == KindId(5) && read.get_entity(edge.source).is_some())
        .map(|edge| (edge.source, row_identity(edge.target)))
        .collect::<BTreeMap<_, _>>();
    let events = read
        .relations()
        .into_iter()
        .filter(|edge| edge.kind.kind_id == KindId(4))
        .filter_map(|edge| {
            let state = edge.authoritative_aspect_state.as_ref().unwrap();
            let ContractValidatedAspectValueView::Struct(fields) =
                state.get(&aspect_key("aggregate-lineage")).unwrap().view()
            else {
                panic!("lineage grammar")
            };
            let AspectValue::UInt64(occurrence) = fields.get(&field_key("occurrence")).unwrap()
            else {
                panic!("occurrence")
            };
            let AspectValue::UInt8(kind) = fields.get(&field_key("kind")).unwrap() else {
                panic!("kind")
            };
            (*occurrence == result.commit.commit_id.0).then(|| {
                (
                    *occurrence,
                    *kind,
                    row_identity(edge.source),
                    row_identity(edge.target),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let captures = control.captures.lock().unwrap();
    assert!(captures.len() >= 4, "both real rules prepare and evaluate");
    assert_eq!(
        captures
            .iter()
            .map(|capture| (capture.rule, capture.evaluation))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([(0, false), (0, true), (1, false), (1, true)])
    );
    let mut observed_events = BTreeSet::new();
    for capture in captures.iter() {
        assert!(capture.errors.is_empty(), "{:?}", capture.errors);
        assert!(!capture.proposed.is_empty());
        for aggregate in &capture.proposed {
            assert_eq!(aggregate.definition(), descriptor().definition);
            let expected = bindings
                .iter()
                .filter_map(|(member, identity)| {
                    (*identity == aggregate.identity()).then_some(*member)
                })
                .collect::<BTreeSet<_>>();
            assert_group(aggregate, aggregate.identity(), &expected);
            assert!(rows
                .iter()
                .any(|(_, row)| row.identity == aggregate.identity()
                    && row.current_anchor == Some(aggregate.anchor())
                    && row.retired.is_none()));
            observed_events.extend(aggregate.transition_lineage().map(|event| {
                (
                    event.occurrence().0,
                    match event.kind() {
                        AggregateLineageKind::SplitFrom => 1,
                        AggregateLineageKind::MergedInto => 2,
                    },
                    event.from(),
                    event.to(),
                )
            }));
        }
        if let Some(before) = before {
            let prior_rows = records(execution, runtime, before);
            let prior = runtime
                .read_truth()
                .read_snapshot(before, execution)
                .unwrap()
                .unwrap();
            for aggregate in &capture.committed {
                assert_eq!(
                    aggregate.identity(),
                    identity(&prior_rows, aggregate.anchor())
                );
                let expected = prior
                    .relations()
                    .into_iter()
                    .filter(|edge| edge.kind.kind_id == KindId(5))
                    .filter(|edge| {
                        prior_rows.iter().any(|(id, row)| {
                            *id == edge.target && row.identity == aggregate.identity()
                        })
                    })
                    .map(|edge| edge.source)
                    .collect::<BTreeSet<_>>();
                assert_group(aggregate, aggregate.identity(), &expected);
                assert_eq!(aggregate.transition_lineage().count(), 0);
            }
            assert!(!capture.committed.is_empty());
        }
    }
    assert_eq!(
        observed_events, events,
        "same actual transition publishes the complete event tuples"
    );
}

fn assert_group(
    aggregate: &AggregateRead,
    identity: AggregateIdentity,
    members: &BTreeSet<EntityId>,
) {
    assert_eq!(aggregate.identity(), identity);
    assert_eq!(aggregate.members().collect::<BTreeSet<_>>(), *members);
    assert_eq!(aggregate.len(), members.len());
    assert_eq!(aggregate.anchor(), *members.first().unwrap());
    assert!(!aggregate.is_empty());
}
