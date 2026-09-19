use std::rc::Rc;

#[cfg(test)]
use std::rc::Weak;

use super::family_index::WorthUiPlanRegionFamilyIndex;
use super::identity_trie::{self, WorthUiPlanRegionIdentityTrieNode};
use super::record::WorthUiPlanRegionRecord;
use super::slot_set::WorthUiPlanRegionSlotSetNode;
use super::slot_trie::{self, WorthUiPlanRegionSlotTrieNode};
use super::{
    WorthUiPlanRegionHandle, WorthUiPlanRegionIdentity, WorthUiPlanRegionMutation,
    WorthUiPlanRegionSchema, WorthUiPlanRegionStorageCounters, WorthUiPlanRegionStoreDenial,
    WorthUiPlanRegionSuccessor, WorthUiPlanRegionTransition, WorthUiPlanRegionTransitionEvidence,
};

#[path = "storage_equivalence.rs"]
mod equivalence;
#[path = "storage_lane_contract.rs"]
mod lane_contract;
#[path = "mounted_projection.rs"]
mod mounted_projection;
#[path = "storage_mutation.rs"]
mod mutation;
#[path = "successor_region_count.rs"]
mod successor_region_count;
use equivalence::schemas_match;
use successor_region_count::expected_region_count_after;

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiPlanRegionStore {
    identity_root: Option<Rc<WorthUiPlanRegionIdentityTrieNode>>,
    slot_root: Option<Rc<WorthUiPlanRegionSlotTrieNode>>,
    family_index: WorthUiPlanRegionFamilyIndex,
    root_shell_root: Option<Rc<WorthUiPlanRegionSlotSetNode>>,
    realtime_budget_exhaustion_root: Option<Rc<WorthUiPlanRegionSlotSetNode>>,
    mounted_projection_index: crate::runtime::persistent_index::UiPersistentOrdMap<
        u64,
        crate::runtime::persistent_index::UiPersistentOrdSet<u32>,
    >,
    mounted_theme_token_index: crate::runtime::persistent_index::UiPersistentOrdMap<
        crate::capability::ThemeTokenId,
        crate::runtime::persistent_index::UiPersistentOrdSet<u32>,
    >,
    root_shell_count: usize,
    region_count: usize,
    next_stable_slot: u64,
    semantic_digest: u64,
}

#[cfg(test)]
pub(crate) struct WorthUiPlanRegionStorageReclamationProbe {
    identity_root: Weak<WorthUiPlanRegionIdentityTrieNode>,
    slot_root: Weak<WorthUiPlanRegionSlotTrieNode>,
    exclusive_family_roots: Vec<Weak<WorthUiPlanRegionSlotSetNode>>,
}

#[cfg(test)]
impl WorthUiPlanRegionStorageReclamationProbe {
    pub(crate) fn is_reclaimed(&self) -> bool {
        self.identity_root.upgrade().is_none()
            && self.slot_root.upgrade().is_none()
            && self
                .exclusive_family_roots
                .iter()
                .all(|root| root.upgrade().is_none())
    }
}

impl WorthUiPlanRegionStore {
    pub(super) fn record_for_identity(
        &self,
        identity: &WorthUiPlanRegionIdentity,
    ) -> Option<&Rc<WorthUiPlanRegionRecord>> {
        identity_trie::lookup(&self.identity_root, identity)
    }

    pub(super) fn next_stable_slot_value(&self) -> u64 {
        self.next_stable_slot
    }

    pub(super) fn advance_stable_slot(
        &mut self,
    ) -> Result<(), crate::runtime::WorthUiHandleCapacityExhaustion> {
        self.next_stable_slot =
            crate::runtime::execution::handle_allocation::WorthUiHandleCapacity::next_stable_slot(
                self.next_stable_slot,
            )?;
        Ok(())
    }

    pub(super) fn increment_region_count(&mut self) {
        self.region_count += 1;
    }

    pub(crate) fn try_launch(
        inputs: impl IntoIterator<Item = crate::runtime::WorthUiPlanNodeInput>,
    ) -> Result<WorthUiPlanRegionSuccessor, WorthUiPlanRegionStoreDenial> {
        let mut store = Self::default();
        let mut counters = WorthUiPlanRegionStorageCounters::default();
        let mut evidence = Vec::new();
        let schemas = inputs
            .into_iter()
            .map(WorthUiPlanRegionSchema::from_node_input)
            .collect::<Vec<_>>();
        super::schema_batch::validate_launch_owner_bundles(&schemas)?;
        store.apply_schema_batch(schemas, &mut evidence, &mut counters)?;
        evidence.sort_by(|left, right| left.region_identity().cmp(right.region_identity()));
        Ok(WorthUiPlanRegionSuccessor {
            store,
            evidence,
            counters,
        })
    }

    #[cfg(test)]
    pub(crate) fn launch(
        inputs: impl IntoIterator<Item = crate::runtime::WorthUiPlanNodeInput>,
    ) -> WorthUiPlanRegionSuccessor {
        Self::try_launch(inputs).expect("test region fixture remains structurally valid")
    }

    pub(crate) fn try_successor(
        &self,
        mut mutations: Vec<WorthUiPlanRegionMutation>,
    ) -> Result<WorthUiPlanRegionSuccessor, WorthUiPlanRegionStoreDenial> {
        mutations.sort_by(|left, right| left.identity().cmp(right.identity()));
        let mut store = self.clone();
        let mut expected_count = store.region_count;
        let mut counters = WorthUiPlanRegionStorageCounters::default();
        let mut evidence = Vec::with_capacity(mutations.len());
        for mutation in mutations {
            expected_count = expected_region_count_after(&store, &mutation, expected_count);
            match mutation {
                WorthUiPlanRegionMutation::Upsert(schema) => {
                    store.upsert(schema, &mut evidence, &mut counters)?;
                }
                WorthUiPlanRegionMutation::Insert(schema) => {
                    store.force_replace(
                        schema,
                        WorthUiPlanRegionTransition::Replaced,
                        &mut evidence,
                        &mut counters,
                    )?;
                }
                WorthUiPlanRegionMutation::Replace(schema) => {
                    store.force_replace(
                        schema,
                        WorthUiPlanRegionTransition::Replaced,
                        &mut evidence,
                        &mut counters,
                    )?;
                }
                WorthUiPlanRegionMutation::Reparent(schema) => {
                    store.force_replace(
                        schema,
                        WorthUiPlanRegionTransition::Reparented,
                        &mut evidence,
                        &mut counters,
                    )?;
                }
                WorthUiPlanRegionMutation::Rebind(schema) => {
                    store.force_replace(
                        schema,
                        WorthUiPlanRegionTransition::Rebound,
                        &mut evidence,
                        &mut counters,
                    )?;
                }
                WorthUiPlanRegionMutation::LaneTransition(schema) => {
                    store.force_replace(
                        schema,
                        WorthUiPlanRegionTransition::LaneTransitioned,
                        &mut evidence,
                        &mut counters,
                    )?;
                }
                WorthUiPlanRegionMutation::Retire(identity) => {
                    store.retire(identity, &mut evidence, &mut counters);
                }
                WorthUiPlanRegionMutation::OwnerBundle { root, schemas } => {
                    store.reconcile_owner_bundle(root, schemas, &mut evidence, &mut counters)?;
                }
                WorthUiPlanRegionMutation::RetireOwner(identity) => {
                    store.retire_owner_bundle(identity, &mut evidence, &mut counters);
                }
            }
        }
        if store.region_count != expected_count {
            return Err(WorthUiPlanRegionStoreDenial::IncompleteSuccessor);
        }
        evidence.sort_by(|left, right| left.region_identity().cmp(right.region_identity()));
        Ok(WorthUiPlanRegionSuccessor {
            store,
            evidence,
            counters,
        })
    }

    #[cfg(test)]
    pub(crate) fn successor(
        &self,
        mutations: Vec<WorthUiPlanRegionMutation>,
    ) -> WorthUiPlanRegionSuccessor {
        self.try_successor(mutations)
            .expect("test regional successor remains structurally valid")
    }

    pub(crate) fn handle_for(
        &self,
        identity: &WorthUiPlanRegionIdentity,
    ) -> Option<&WorthUiPlanRegionHandle> {
        identity_trie::lookup(&self.identity_root, identity).map(|record| &record.handle)
    }

    pub(crate) fn handle_for_stable_slot(
        &self,
        stable_slot: u64,
    ) -> Option<&WorthUiPlanRegionHandle> {
        slot_trie::lookup(&self.slot_root, stable_slot).map(|record| &record.handle)
    }

    pub(crate) fn executable_for(
        &self,
        identity: &WorthUiPlanRegionIdentity,
    ) -> Option<&super::WorthUiPlanRegionExecutable> {
        identity_trie::lookup(&self.identity_root, identity).map(|record| &record.executable)
    }

    pub(crate) fn executable_for_stable_slot(
        &self,
        stable_slot: u64,
    ) -> Option<&super::WorthUiPlanRegionExecutable> {
        slot_trie::lookup(&self.slot_root, stable_slot).map(|record| &record.executable)
    }

    pub(crate) fn runtime_handle_for_stable_slot(
        &self,
        stable_slot: u64,
        arena_identity: crate::runtime::WorthUiHandleArenaIdentity,
    ) -> Option<crate::runtime::WorthUiRuntimeHandle> {
        let record = slot_trie::lookup(&self.slot_root, stable_slot)?;
        let plan_index = u32::try_from(stable_slot).ok()?;
        Some(crate::runtime::WorthUiRuntimeHandle::new(
            record.executable.family(),
            plan_index,
            crate::runtime::WorthUiHandleSlotGeneration::new(record.handle.slot_generation()),
            arena_identity,
        ))
    }

    pub(crate) fn family_count(&self, family: crate::runtime::WorthUiPlanNodeInputFamily) -> usize {
        self.family_index.count(family)
    }

    pub(crate) fn family_semantic_digest(
        &self,
        family: crate::runtime::WorthUiPlanNodeInputFamily,
    ) -> u64 {
        self.family_index.semantic_digest(family)
    }

    pub(crate) fn family_slot_view<const N: usize>(
        &self,
        families: [crate::runtime::WorthUiPlanNodeInputFamily; N],
    ) -> super::WorthUiPlanRegionSlotSetView<N> {
        self.family_index.view(families)
    }

    pub(crate) fn root_shell_slot_view(&self) -> super::WorthUiPlanRegionSlotSetView<1> {
        super::WorthUiPlanRegionSlotSetView::new(
            [self.root_shell_root.clone()],
            self.root_shell_count,
        )
    }

    #[cfg(test)]
    pub(crate) fn schema_for(
        &self,
        identity: &WorthUiPlanRegionIdentity,
    ) -> Option<&WorthUiPlanRegionSchema> {
        identity_trie::lookup(&self.identity_root, identity).map(|record| &record.schema)
    }

    #[cfg(test)]
    pub(crate) fn resolves(&self, handle: &WorthUiPlanRegionHandle) -> bool {
        self.handle_for_stable_slot(handle.stable_slot()) == Some(handle)
    }

    #[cfg(test)]
    pub(crate) fn shares_exact_region_storage_with(
        &self,
        other: &Self,
        identity: &WorthUiPlanRegionIdentity,
    ) -> bool {
        match (
            identity_trie::lookup(&self.identity_root, identity),
            identity_trie::lookup(&other.identity_root, identity),
        ) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }

    pub(crate) fn region_count(&self) -> usize {
        self.region_count
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }

    pub(crate) fn canonical_identities(&self) -> Vec<WorthUiPlanRegionIdentity> {
        let mut records = Vec::with_capacity(self.region_count);
        identity_trie::collect_records(&self.identity_root, &mut records);
        records
            .into_iter()
            .map(|record| record.schema.identity().clone())
            .collect()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn reconstructive_inspection_rows(
        &self,
        arena_identity: crate::runtime::WorthUiHandleArenaIdentity,
    ) -> Vec<(
        crate::runtime::WorthUiPlanNode,
        crate::runtime::WorthUiPlanNodeInput,
        crate::runtime::WorthUiPlanExecutionLane,
    )> {
        let mut records = Vec::with_capacity(self.region_count);
        slot_trie::collect_records(&self.slot_root, &mut records);
        records
            .into_iter()
            .map(|record| {
                let plan_index = u32::try_from(record.handle.stable_slot())
                    .expect("sealed regional slots satisfy compact inspection capacity");
                let runtime_handle = crate::runtime::WorthUiRuntimeHandle::new(
                    record.executable.family(),
                    plan_index,
                    crate::runtime::WorthUiHandleSlotGeneration::new(
                        record.handle.slot_generation(),
                    ),
                    arena_identity,
                );
                let child_range = record.executable.child_range_for_plan_index(plan_index);
                (
                    record
                        .executable
                        .materialize_node(runtime_handle, child_range),
                    record.schema.input().clone(),
                    record.executable.lane(),
                )
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn retained_storage_node_count(&self) -> usize {
        identity_trie::reachable_node_count(&self.identity_root)
            + slot_trie::reachable_node_count(&self.slot_root)
            + self.family_index.reachable_node_count()
            + super::slot_set::reachable_node_count(&self.root_shell_root)
            + super::slot_set::reachable_node_count(&self.realtime_budget_exhaustion_root)
    }

    #[cfg(test)]
    pub(crate) fn reclamation_probe(
        &self,
        include_exclusive_family_roots: bool,
    ) -> WorthUiPlanRegionStorageReclamationProbe {
        WorthUiPlanRegionStorageReclamationProbe {
            identity_root: self
                .identity_root
                .as_ref()
                .map_or_else(Weak::new, Rc::downgrade),
            slot_root: self
                .slot_root
                .as_ref()
                .map_or_else(Weak::new, Rc::downgrade),
            exclusive_family_roots: if include_exclusive_family_roots {
                let mut roots = self.family_index.exclusive_root_probes();
                if let Some(root) = &self.realtime_budget_exhaustion_root {
                    if Rc::strong_count(root) == 1 {
                        roots.push(Rc::downgrade(root));
                    }
                }
                roots
            } else {
                Vec::new()
            },
        }
    }
}
