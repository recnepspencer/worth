use super::*;
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::graph::SignalGraph;

#[test]
fn admitted_normalization_matches_independent_sort_for_duplicates_and_scope_payloads() {
    let mut graph = SignalGraph::new();
    let nodes = [
        graph.node().build(),
        graph.node().build(),
        graph.node().build(),
    ];
    for count in 0..96 {
        let entries = (0..count)
            .rev()
            .map(|index| DependencySnapshotEntry {
                source: nodes[index % nodes.len()],
                aspect: Aspect::new((index % 4) as u8),
                cached_version: 0,
                scope: (index % 3 != 0).then(|| {
                    PartitionSubscription::partition_and_detail(
                        format!("partition-{}", index % 5),
                        format!("detail-λ-{}", index % 2),
                    )
                }),
            })
            .collect::<Vec<_>>();
        let mask = AspectMask::from_aspect(Aspect::new(1));
        let mut expected = vec![(nodes[0], Aspect::new(1), None)];
        expected.extend(
            entries
                .iter()
                .map(|entry| (entry.source, entry.aspect, entry.scope.clone())),
        );
        expected.sort_by_key(|(node, aspect, scope)| {
            (
                node.index(),
                node.generation(),
                aspect.index(),
                scope.clone(),
            )
        });
        expected.dedup();
        let mut full = RetainedStoragePreparation::new(1_000_000);
        assert_eq!(
            collect(nodes[0], mask, &entries, &mut full).unwrap(),
            expected
        );
        assert_eq!(
            collect(
                nodes[0],
                mask,
                &entries,
                &mut RetainedStoragePreparation::new(full.visits())
            )
            .unwrap(),
            expected
        );
        let limit = full.visits() - 1;
        assert_eq!(
            collect(
                nodes[0],
                mask,
                &entries,
                &mut RetainedStoragePreparation::new(limit)
            ),
            Err(SignalError::ConditionalEvaluationWorkExhausted {
                maximum_visits: limit
            })
        );
    }
}

#[test]
fn equal_edge_counts_cannot_hide_large_scope_payload_traversal() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    for detail in [String::new(), "λ".repeat(8_192)] {
        let large = !detail.is_empty();
        let entries = [DependencySnapshotEntry {
            source: node,
            aspect: Aspect::new(0),
            cached_version: 0,
            scope: Some(PartitionSubscription::partition_and_detail(
                "partition",
                detail,
            )),
        }];
        let mut work = RetainedStoragePreparation::new(1_024);
        let result = collect(node, AspectMask::EMPTY, &entries, &mut work);
        if large {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: 1_024
                })
            );
            assert_eq!(
                work.visits(),
                MAX_ASPECTS + 1,
                "denial occurs after length inspection, before scope cloning"
            );
        } else {
            assert_eq!(result.unwrap().len(), 1);
        }
    }
    assert!(normalization_bound(usize::MAX, 0, 0).is_none());
    assert!(normalization_bound(2, 0, usize::MAX).is_none());
}
