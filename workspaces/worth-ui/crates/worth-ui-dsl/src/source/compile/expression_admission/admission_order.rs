use std::collections::BTreeMap;

use super::pending_expression::PendingExpression;
use crate::WorthUiExpressionOperandSource;

/// The expression-to-expression references of one package.
///
/// Nodes are the pending expressions in identity order; an edge runs from an
/// expression to each `condition` or `derived` operand source it reads.
/// Unknown targets have no edge; operand resolution reports them.
pub(super) struct ExpressionGraph {
    edges: Vec<Vec<usize>>,
}

impl ExpressionGraph {
    pub(super) fn new(pending: &[PendingExpression]) -> Self {
        let index: BTreeMap<&str, usize> = pending
            .iter()
            .enumerate()
            .map(|(position, expression)| (expression.declaration.identity(), position))
            .collect();
        let edges = pending
            .iter()
            .map(|expression| {
                expression
                    .declaration
                    .operands()
                    .iter()
                    .filter_map(|operand| match operand.source() {
                        WorthUiExpressionOperandSource::Condition { identity }
                        | WorthUiExpressionOperandSource::Derived { identity } => {
                            index.get(identity.as_str()).copied()
                        }
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        Self { edges }
    }

    /// Every cycle found by a depth-first walk in identity order, each as the
    /// node positions from its entry around the loop. A self-reference is a
    /// cycle of one.
    pub(super) fn cycles(&self) -> Vec<Vec<usize>> {
        let mut state = vec![Visit::Unseen; self.edges.len()];
        let mut cycles = Vec::new();
        for root in 0..self.edges.len() {
            if state[root] == Visit::Unseen {
                self.walk(root, &mut state, &mut cycles);
            }
        }
        cycles
    }

    fn walk(&self, root: usize, state: &mut [Visit], cycles: &mut Vec<Vec<usize>>) {
        let mut path: Vec<(usize, usize)> = vec![(root, 0)];
        state[root] = Visit::OnPath;
        while let Some(&(node, next_edge)) = path.last() {
            let Some(&target) = self.edges[node].get(next_edge) else {
                state[node] = Visit::Done;
                path.pop();
                continue;
            };
            if let Some(top) = path.last_mut() {
                top.1 += 1;
            }
            match state[target] {
                Visit::Unseen => {
                    state[target] = Visit::OnPath;
                    path.push((target, 0));
                }
                Visit::OnPath => {
                    let entry = path
                        .iter()
                        .position(|(on_path, _)| *on_path == target)
                        .expect("a node marked on-path is on the path");
                    cycles.push(path[entry..].iter().map(|(node, _)| *node).collect());
                }
                Visit::Done => {}
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Visit {
    Unseen,
    OnPath,
    Done,
}
