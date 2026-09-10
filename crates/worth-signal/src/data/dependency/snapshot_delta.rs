use serde::{Deserialize, Serialize};

use crate::data::handle::NodeId;

use super::{DependencySnapshot, SharedDependencySnapshot, SnapshotChangeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDeltaRecord {
    pub node: NodeId,
    pub change_kind: SnapshotChangeKind,
    pub previous_entry_count: u32,
    pub next_entry_count: u32,
    pub changed_entry_count: u32,
}

impl SnapshotDeltaRecord {
    pub fn between(
        node: NodeId,
        previous: &DependencySnapshot,
        next: &SharedDependencySnapshot,
    ) -> Self {
        let previous_entries = previous.entries();
        let next_entries = next.entries();
        let mut changed_entry_count = 0_u32;
        let mut previous_index = 0usize;
        let mut next_index = 0usize;

        while previous_index < previous_entries.len() && next_index < next_entries.len() {
            let previous_entry = &previous_entries[previous_index];
            let next_entry = &next_entries[next_index];
            match previous_entry.compare_key(next_entry) {
                std::cmp::Ordering::Less => {
                    changed_entry_count += 1;
                    previous_index += 1;
                }
                std::cmp::Ordering::Greater => {
                    changed_entry_count += 1;
                    next_index += 1;
                }
                std::cmp::Ordering::Equal => {
                    if previous_entry.cached_version != next_entry.cached_version {
                        changed_entry_count += 1;
                    }
                    previous_index += 1;
                    next_index += 1;
                }
            }
        }

        changed_entry_count += (previous_entries.len() - previous_index) as u32;
        changed_entry_count += (next_entries.len() - next_index) as u32;

        Self {
            node,
            change_kind: if changed_entry_count == 0 && previous_entries.len() == next_entries.len()
            {
                SnapshotChangeKind::Unchanged
            } else {
                SnapshotChangeKind::StructuralReplace
            },
            previous_entry_count: previous_entries.len() as u32,
            next_entry_count: next_entries.len() as u32,
            changed_entry_count,
        }
    }

    pub fn changed(&self) -> bool {
        self.changed_entry_count > 0 || self.previous_entry_count != self.next_entry_count
    }

    pub fn for_version_update(
        node: NodeId,
        previous: &DependencySnapshot,
        cached_versions: &[u64],
    ) -> Self {
        debug_assert_eq!(previous.entries().len(), cached_versions.len());
        Self {
            node,
            change_kind: if previous
                .entries()
                .iter()
                .zip(cached_versions.iter().copied())
                .all(|(entry, cached_version)| entry.cached_version == cached_version)
            {
                SnapshotChangeKind::Unchanged
            } else {
                SnapshotChangeKind::StableShapeVersionOnly
            },
            previous_entry_count: previous.entries().len() as u32,
            next_entry_count: cached_versions.len() as u32,
            changed_entry_count: previous
                .entries()
                .iter()
                .zip(cached_versions.iter().copied())
                .filter(|(entry, cached_version)| entry.cached_version != *cached_version)
                .count() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::Aspect;

    #[test]
    fn version_update_delta_marks_stable_shape_change_kind() {
        let source = NodeId::new(1, 0);
        let mut snapshot = DependencySnapshot::empty();
        snapshot.record(source, Aspect::new(0), 5, None);
        snapshot.record(source, Aspect::new(1), 9, None);

        let delta = SnapshotDeltaRecord::for_version_update(NodeId::new(0, 0), &snapshot, &[7, 9]);
        assert_eq!(
            delta.change_kind,
            SnapshotChangeKind::StableShapeVersionOnly
        );
        assert!(delta.changed());
    }
}
