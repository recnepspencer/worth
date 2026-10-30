use crate::data::dependency::DependencyEdge;
use crate::logic::explain::{RewiringDependency, RewiringSummary};

pub(super) fn count_dependency_updates(
    current_dependencies: &[DependencyEdge],
    next_dependencies: &[DependencyEdge],
) -> u32 {
    let mut current_index = 0usize;
    let mut next_index = 0usize;
    let mut changes = 0u32;

    while current_index < current_dependencies.len() && next_index < next_dependencies.len() {
        match compare_dependency_edges(
            &current_dependencies[current_index],
            &next_dependencies[next_index],
        ) {
            std::cmp::Ordering::Less => {
                changes += 1;
                current_index += 1;
            }
            std::cmp::Ordering::Greater => {
                changes += 1;
                next_index += 1;
            }
            std::cmp::Ordering::Equal => {
                current_index += 1;
                next_index += 1;
            }
        }
    }

    changes
        + (current_dependencies.len() - current_index) as u32
        + (next_dependencies.len() - next_index) as u32
}

pub(super) fn rewiring_summary_from_lowered_edges(
    current_dependencies: &[DependencyEdge],
    next_dependencies: &[DependencyEdge],
) -> Option<RewiringSummary> {
    let mut current_index = 0usize;
    let mut next_index = 0usize;
    let mut added = Vec::new();
    let mut removed = Vec::new();

    while current_index < current_dependencies.len() && next_index < next_dependencies.len() {
        match compare_dependency_edges(
            &current_dependencies[current_index],
            &next_dependencies[next_index],
        ) {
            std::cmp::Ordering::Less => {
                let edge = &current_dependencies[current_index];
                removed.push(RewiringDependency {
                    source: edge.source(),
                    aspect: edge.aspect(),
                    subscription: edge.scope_ref().cloned(),
                });
                current_index += 1;
            }
            std::cmp::Ordering::Greater => {
                let edge = &next_dependencies[next_index];
                added.push(RewiringDependency {
                    source: edge.source(),
                    aspect: edge.aspect(),
                    subscription: edge.scope_ref().cloned(),
                });
                next_index += 1;
            }
            std::cmp::Ordering::Equal => {
                current_index += 1;
                next_index += 1;
            }
        }
    }

    while current_index < current_dependencies.len() {
        let edge = &current_dependencies[current_index];
        removed.push(RewiringDependency {
            source: edge.source(),
            aspect: edge.aspect(),
            subscription: edge.scope_ref().cloned(),
        });
        current_index += 1;
    }

    while next_index < next_dependencies.len() {
        let edge = &next_dependencies[next_index];
        added.push(RewiringDependency {
            source: edge.source(),
            aspect: edge.aspect(),
            subscription: edge.scope_ref().cloned(),
        });
        next_index += 1;
    }

    if added.is_empty() && removed.is_empty() {
        None
    } else {
        added.sort_by_key(|dependency| {
            (
                dependency.source.index(),
                dependency.source.generation(),
                dependency.aspect.index(),
            )
        });
        removed.sort_by_key(|dependency| {
            (
                dependency.source.index(),
                dependency.source.generation(),
                dependency.aspect.index(),
            )
        });
        Some(RewiringSummary { added, removed })
    }
}

fn compare_dependency_edges(left: &DependencyEdge, right: &DependencyEdge) -> std::cmp::Ordering {
    (
        left.source().index(),
        left.source().generation(),
        left.aspect().index(),
        left.scope_ref(),
    )
        .cmp(&(
            right.source().index(),
            right.source().generation(),
            right.aspect().index(),
            right.scope_ref(),
        ))
}
