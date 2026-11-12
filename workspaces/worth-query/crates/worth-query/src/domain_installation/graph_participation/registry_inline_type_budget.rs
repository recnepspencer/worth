//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryInstalledGraphParticipationRegistry>("workspaces/worth-query/crates/worth-query/src/domain_installation/graph_participation/registry.rs", "WorthQueryInstalledGraphParticipationRegistry", 128 * 1024);
        owner.field::<HashMap<TypeId, Arc<WorthQueryInstalledGraphParticipationRecord>>>("records");
    }
}
