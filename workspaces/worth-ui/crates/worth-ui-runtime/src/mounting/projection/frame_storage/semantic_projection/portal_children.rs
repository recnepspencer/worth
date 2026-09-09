use super::{UiMountedProjectionNodeRecord, UiMountedSemanticProjection};
use crate::capability::ComponentId;
use crate::runtime::persistent_index::{
    UiPersistentIndexMutationWork, UiPersistentOrdMap, UiPersistentOrdSet,
};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

type OwnerKey = (UiSemanticSurfaceIdentity, ComponentId);

/// Derived membership in an authored Portal child neighborhood. It grants no
/// Portal lifecycle or mounted receipt authority; the semantic node owns the relation.
#[derive(Clone, Default)]
pub(super) struct UiMountedPortalChildMembership {
    by_owner: UiPersistentOrdMap<OwnerKey, UiPersistentOrdSet<UiMountedInstanceIdentity>>,
    member_bytes: usize,
    owner_text_bytes: usize,
}

impl UiMountedPortalChildMembership {
    pub(super) fn replace(
        &mut self,
        previous: Option<&UiMountedProjectionNodeRecord>,
        successor: Option<&UiMountedProjectionNodeRecord>,
    ) -> UiPersistentIndexMutationWork {
        let previous = previous.and_then(membership);
        let successor = successor.and_then(membership);
        let mut work = UiPersistentIndexMutationWork::default();
        if previous == successor {
            return work;
        }
        if let Some((key, instance)) = previous {
            work.merge(self.update(key, instance, false))
                .expect("membership work fits address space");
        }
        if let Some((key, instance)) = successor {
            work.merge(self.update(key, instance, true))
                .expect("membership work fits address space");
        }
        work
    }

    fn update(
        &mut self,
        key: OwnerKey,
        instance: UiMountedInstanceIdentity,
        insert: bool,
    ) -> UiPersistentIndexMutationWork {
        let (members, probes) = self.by_owner.get_with_probes(&key);
        let had_members = members.is_some();
        let mut members = members.cloned().unwrap_or_default();
        let mut work = UiPersistentIndexMutationWork::with_key_probes(probes);
        let previous_bytes = members
            .retained_structural_bytes()
            .expect("membership bytes fit address space");
        let changed = if insert {
            members.insert_with_work(instance)
        } else {
            members.remove_with_work(&instance)
        };
        work.merge(changed.1)
            .expect("membership work fits address space");
        if !changed.0 {
            return work;
        }
        if !had_members {
            self.owner_text_bytes = self
                .owner_text_bytes
                .checked_add(key.1.as_str().len())
                .expect("owner identity bytes fit address space");
        }
        if members.is_empty() {
            self.owner_text_bytes = self
                .owner_text_bytes
                .checked_sub(key.1.as_str().len())
                .expect("removed owner identity was retained");
        }
        self.member_bytes = self
            .member_bytes
            .checked_sub(previous_bytes)
            .and_then(|bytes| bytes.checked_add(members.retained_structural_bytes()?))
            .expect("membership bytes fit address space");
        let mutation = if members.is_empty() {
            self.by_owner.remove_with_work(&key).1
        } else {
            self.by_owner.insert_with_work(key, members)
        };
        work.merge(mutation)
            .expect("membership work fits address space");
        work
    }

    pub(super) fn retained_structural_bytes(&self) -> Option<usize> {
        self.by_owner
            .retained_structural_bytes()?
            .checked_add(self.member_bytes)?
            .checked_add(self.owner_text_bytes)
    }
}

fn membership(
    node: &UiMountedProjectionNodeRecord,
) -> Option<(OwnerKey, UiMountedInstanceIdentity)> {
    Some((
        (
            node.receipt.semantic_surface(),
            node.portal_child_owner.clone()?,
        ),
        node.receipt.mounted_instance(),
    ))
}

impl UiMountedSemanticProjection {
    /// Exact related instances plus actual index probes and visited memberships.
    /// Declaration identity alone cannot join children across semantic surfaces.
    pub(in crate::mounting::projection) fn portal_children_for_owners(
        &self,
        owners: &[UiMountedInstanceIdentity],
    ) -> (Vec<UiMountedInstanceIdentity>, usize) {
        let mut keys = std::collections::BTreeSet::new();
        let mut work = 0usize;
        for owner in owners {
            let (node, probes) = self.nodes.get_with_probes(owner);
            work += probes;
            if let Some(node) = node {
                if let Some(component) = &node.component_id {
                    keys.insert((node.receipt.semantic_surface(), component.clone()));
                }
            }
        }
        let mut selected = Vec::new();
        for key in keys {
            let (members, probes) = self.portal_children.by_owner.get_with_probes(&key);
            work += probes;
            if let Some(members) = members {
                for instance in members.iter().copied() {
                    work += 1;
                    selected.push(instance);
                }
            }
        }
        selected.sort_unstable();
        selected.dedup();
        (selected, work)
    }
}
