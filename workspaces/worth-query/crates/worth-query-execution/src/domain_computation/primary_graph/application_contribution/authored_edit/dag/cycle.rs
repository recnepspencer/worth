//! Extract an actual directed cycle, excluding merely blocked descendants.
use super::*;
pub(super) fn find(count: usize, edges: &[(usize, usize, DeclaredEditDependency)]) -> Vec<usize> {
    let mut outgoing = vec![Vec::new(); count];
    for (index, (writer, _, _)) in edges.iter().enumerate() {
        outgoing[*writer].push(index);
    }
    let mut color = vec![0u8; count];
    let mut path = Vec::new();
    for start in 0..count {
        if color[start] != 0 {
            continue;
        }
        let mut stack = vec![(start, 0usize)];
        while let Some((node, next)) = stack.last_mut() {
            if color[*node] == 0 {
                color[*node] = 1;
                path.push(*node);
            }
            if let Some(edge) = outgoing[*node].get(*next) {
                *next += 1;
                let target = edges[*edge].1;
                match color[target] {
                    0 => stack.push((target, 0)),
                    1 => {
                        return path[path
                            .iter()
                            .position(|node| *node == target)
                            .expect("active node is in the path")..]
                            .to_vec()
                    }
                    _ => {}
                }
            } else {
                color[*node] = 2;
                path.pop();
                stack.pop();
            }
        }
    }
    unreachable!("a finite graph rejected by topological ordering has a directed cycle")
}
