use serde::{Deserialize, Serialize};

use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::handle::NodeId;

use super::read_set::AdmittedHostComputedReadSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostComputedDependencyPatch {
    node: NodeId,
    previous_dependencies: CanonicalDependencies,
    next_dependencies: CanonicalDependencies,
    added_dependencies: CanonicalDependencies,
    removed_dependencies: CanonicalDependencies,
    retained_dependency_count: u32,
}

impl HostComputedDependencyPatch {
    pub fn between(
        node: NodeId,
        previous: &[DependencyEdge],
        next: &AdmittedHostComputedReadSet,
    ) -> Self {
        let previous_dependencies = CanonicalDependencies::from_slice(previous);
        let next_dependencies = next.canonical_dependencies().clone();
        let mut previous_index = 0usize;
        let mut next_index = 0usize;
        let previous_slice = previous_dependencies.as_slice();
        let next_slice = next_dependencies.as_slice();
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut retained_dependency_count = 0u32;

        while previous_index < previous_slice.len() && next_index < next_slice.len() {
            let previous_dependency = &previous_slice[previous_index];
            let next_dependency = &next_slice[next_index];
            match previous_dependency
                .sort_key()
                .cmp(&next_dependency.sort_key())
            {
                std::cmp::Ordering::Less => {
                    removed.push(previous_dependency.clone());
                    previous_index += 1;
                }
                std::cmp::Ordering::Greater => {
                    added.push(next_dependency.clone());
                    next_index += 1;
                }
                std::cmp::Ordering::Equal => {
                    retained_dependency_count += 1;
                    previous_index += 1;
                    next_index += 1;
                }
            }
        }

        removed.extend_from_slice(&previous_slice[previous_index..]);
        added.extend_from_slice(&next_slice[next_index..]);

        Self {
            node,
            previous_dependencies,
            next_dependencies,
            added_dependencies: CanonicalDependencies::from_ordered_unique(added),
            removed_dependencies: CanonicalDependencies::from_ordered_unique(removed),
            retained_dependency_count,
        }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn previous_dependencies(&self) -> &[DependencyEdge] {
        self.previous_dependencies.as_slice()
    }

    pub fn next_dependencies(&self) -> &[DependencyEdge] {
        self.next_dependencies.as_slice()
    }

    pub fn added_dependencies(&self) -> &[DependencyEdge] {
        self.added_dependencies.as_slice()
    }

    pub fn removed_dependencies(&self) -> &[DependencyEdge] {
        self.removed_dependencies.as_slice()
    }

    pub fn retained_dependency_count(&self) -> u32 {
        self.retained_dependency_count
    }

    pub fn changed(&self) -> bool {
        !self.added_dependencies.is_empty() || !self.removed_dependencies.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::aspect::Aspect;
    use crate::logic::prepared::PreparedDependencyCapture;

    use super::*;

    #[test]
    fn tracks_added_removed_and_retained_edges() {
        let node = NodeId::new(8, 0);
        let retained_source = NodeId::new(1, 0);
        let removed_source = NodeId::new(2, 0);
        let added_source = NodeId::new(3, 0);
        let mut capture = PreparedDependencyCapture::new();
        capture.record(retained_source, Aspect::new(0), None);
        capture.record(added_source, Aspect::new(1), None);
        let admitted = AdmittedHostComputedReadSet::admit(node, capture).unwrap();

        let patch = HostComputedDependencyPatch::between(
            node,
            &[
                DependencyEdge::new(retained_source, Aspect::new(0)),
                DependencyEdge::new(removed_source, Aspect::new(0)),
            ],
            &admitted,
        );

        assert!(patch.changed());
        assert_eq!(patch.retained_dependency_count(), 1);
        assert_eq!(patch.added_dependencies().len(), 1);
        assert_eq!(patch.added_dependencies()[0].source(), added_source);
        assert_eq!(patch.removed_dependencies().len(), 1);
        assert_eq!(patch.removed_dependencies()[0].source(), removed_source);
    }
}
