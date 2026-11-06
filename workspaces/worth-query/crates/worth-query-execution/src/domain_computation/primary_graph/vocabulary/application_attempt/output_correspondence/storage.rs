//! One sorted immutable role store, shared by correspondence clones.
use super::CommittedOutputBinding;
use crate::domain_computation::primary_graph::application_checkpoint::CheckpointMetadataFunding;

#[derive(Debug, Default)]
pub(super) struct CorrespondenceStorage {
    pub(super) optional_roles: Vec<String>,
    pub(super) roles: Vec<(String, CommittedOutputBinding)>,
    // Last to drop: the rows and their strings retain their actual admission.
    pub(super) _funding: Option<CheckpointMetadataFunding>,
}
impl PartialEq for CorrespondenceStorage {
    fn eq(&self, other: &Self) -> bool {
        self.optional_roles == other.optional_roles && self.roles == other.roles
    }
}
impl Eq for CorrespondenceStorage {}
impl CorrespondenceStorage {
    // The candidate consumes ordered B-tree iterators directly into these rows.
    pub(super) fn from_committed(
        optional_roles: Vec<String>,
        roles: Vec<(String, CommittedOutputBinding)>,
    ) -> Self {
        Self {
            optional_roles,
            roles,
            _funding: None,
        }
    }
    pub(super) fn binding(&self, name: &str) -> Option<&CommittedOutputBinding> {
        self.roles
            .binary_search_by(|(role, _)| role.as_str().cmp(name))
            .ok()
            .map(|slot| &self.roles[slot].1)
    }
}
