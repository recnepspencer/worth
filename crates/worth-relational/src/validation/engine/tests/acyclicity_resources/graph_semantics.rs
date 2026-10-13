//! Independent indegree elimination checks native reachability's cycle verdict.
use super::fixture::*;
use crate::transactions::data::{ConflictClass, TransactionCommitError};
use crate::validation::data::InvariantViolationFields;
use worth_execution::{CancellationToken, ExecutionRequest};

#[test]
fn native_cycle_verdict_agrees_with_topological_elimination_including_convergent_paths_and_loops() {
    let fixture = DagFixture::new(3);
    let before = fixture.head();
    let possible = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
    let mut graphs: Vec<Vec<(usize, usize)>> = (1..1 << possible.len())
        .map(|mask| {
            possible
                .iter()
                .enumerate()
                .filter_map(|(index, &edge)| (mask & (1 << index) != 0).then_some(edge))
                .collect()
        })
        .collect();
    graphs.extend([vec![(0, 0)], vec![(1, 1), (1, 2)]]);
    for edges in graphs {
        let expected_cycle = has_cycle(3, &edges);
        let lease = request(1024 * 1024, 4_000_000, CancellationToken::new(), None);
        let outcome = fixture.validate(
            &ExecutionRequest::leased(&lease),
            fixture.graph_batch(&edges),
        );
        match outcome {
            Ok(candidate) => {
                assert!(
                    !expected_cycle,
                    "native validation accepted cyclic edges {edges:?}"
                );
                drop(candidate);
            }
            Err(TransactionCommitError::Conflict { error, .. }) => {
                assert!(
                    expected_cycle,
                    "native validation refused DAG {edges:?}: {error:?}"
                );
                assert!(
                    matches!(
                        error.class,
                        ConflictClass::InvariantViolation {
                            fields: InvariantViolationFields::Acyclicity { .. },
                            ..
                        }
                    ),
                    "wrong refusal for {edges:?}: {error:?}"
                );
            }
            Err(other) => panic!("setup/resources must not masquerade as a cycle: {other:?}"),
        }
        assert_eq!(fixture.head(), before);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

fn has_cycle(vertices: usize, edges: &[(usize, usize)]) -> bool {
    let mut indegree = vec![0usize; vertices];
    for &(_, target) in edges {
        indegree[target] += 1;
    }
    let mut ready: Vec<_> = (0..vertices)
        .filter(|&vertex| indegree[vertex] == 0)
        .collect();
    let mut removed = 0;
    while let Some(source) = ready.pop() {
        removed += 1;
        for &(_, target) in edges.iter().filter(|(from, _)| *from == source) {
            indegree[target] -= 1;
            if indegree[target] == 0 {
                ready.push(target);
            }
        }
    }
    removed != vertices
}
