use super::super::appearance_state_retirement::UiMountedAppearanceRetirements;
use super::UiMountedAppearanceMembershipWork;
use super::{UiMountedAppearanceStateMembers, UiMountedAppearanceStateMembership};
use crate::runtime::persistent_index::UiPersistentOrdMap;

impl UiMountedAppearanceStateMembers {
    pub(in crate::mounting::projection::frame_storage) fn forget_surface(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiMountedAppearanceMembershipWork {
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(self.primary.len());
        let removed = self
            .primary
            .iter()
            .filter_map(|(key, _)| (key.surface == surface).then_some(key.clone()))
            .collect::<Vec<_>>();
        for key in removed {
            let (_, mutation) = self.remove(&key);
            work.merge(mutation);
        }
        work.add_traversal(self.pending_keys.len());
        self.pending_keys.retain(|key| key.surface != surface);
        work
    }

    pub(in crate::mounting::projection::frame_storage) fn retire_unattached(
        &mut self,
        graph: crate::graph::UiGraphAuthority<'_>,
        retirements: &mut UiMountedAppearanceRetirements,
    ) -> (usize, UiMountedAppearanceMembershipWork) {
        let snapshot = graph.snapshot();
        let mut work = UiMountedAppearanceMembershipWork::default();
        let detached = self
            .primary
            .iter()
            .filter_map(|(key, _)| {
                work.add_traversal(1);
                work.add_lookup(1);
                let attached = snapshot
                    .core_indexes()
                    .node_identity()
                    .node(snapshot.nodes(), key.graph_node)
                    .is_some_and(|node| node.appearance_role_attachment().is_some());
                (!attached).then_some(key.mounted_instance)
            })
            .collect::<Vec<_>>();
        let (retired, retirement_work) = self.retire_instances(&detached, retirements);
        work.merge(retirement_work);
        (retired, work)
    }

    pub(in crate::mounting::projection::frame_storage) fn clear_for_epoch(
        &mut self,
    ) -> (usize, UiMountedAppearanceMembershipWork) {
        let retired = self.primary.len();
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(retired);
        // An epoch invalidates semantic provenance, not the accepted physical
        // output of still-mounted instances. Keep their capacity and identity.
        let physical = self
            .primary
            .iter()
            .filter_map(|(_, membership)| membership.clone().into_predecessor()?.into_physical())
            .collect::<Vec<_>>();
        self.primary = UiPersistentOrdMap::default();
        self.reverse = UiPersistentOrdMap::default();
        self.pending_keys.clear();
        for predecessor in physical {
            let (inserted, mutation) = self.insert_membership(
                predecessor.key.clone(),
                UiMountedAppearanceStateMembership::PhysicalOnly(predecessor),
            );
            work.merge(mutation);
            inserted.expect("epoch succession preserves exact mounted identity");
        }
        (retired, work)
    }

    pub(in crate::mounting::projection::frame_storage) fn retire_instances(
        &mut self,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
        retirements: &mut UiMountedAppearanceRetirements,
    ) -> (usize, UiMountedAppearanceMembershipWork) {
        let mut retired = 0;
        let mut work = UiMountedAppearanceMembershipWork::default();
        for instance in instances {
            let (local_key, probes) = self.reverse.get_with_probes(instance);
            work.add_lookup(probes);
            let Some(local_key) = local_key.cloned() else {
                continue;
            };
            let (membership, removed) = self.remove(&local_key);
            work.merge(removed);
            if let Some(membership) = membership {
                retired += 1;
                if let Some(physical) = membership
                    .into_predecessor()
                    .and_then(|predecessor| predecessor.into_physical())
                {
                    work.merge(retirements.capture(physical));
                }
            }
        }
        (retired, work)
    }
}
