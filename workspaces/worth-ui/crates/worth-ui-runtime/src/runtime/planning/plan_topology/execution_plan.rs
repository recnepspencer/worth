use crate::runtime::{
    WorthUiPlanLanePartition, WorthUiPlanLookupIndex, WorthUiPlanTopology,
    WorthUiPlanTopologyCounters, WorthUiRuntimeHandleAllocationReceipt,
};

#[derive(Clone, Debug)]
pub struct WorthUiExecutionPlan {
    lowering_identity: crate::runtime::planning::WorthUiExecutionPlanLoweringIdentity,
    handle_receipt: WorthUiRuntimeHandleAllocationReceipt,
    flat_projection: Option<super::WorthUiPlanFlatProjection>,
    region_store: super::WorthUiPlanRegionStore,
    construction_counters: super::WorthUiPlanConstructionCounters,
    regional_evidence: super::WorthUiPlanRegionalEvidence,
    counters: WorthUiPlanTopologyCounters,
}

pub(crate) struct WorthUiExecutionPlanConstruction {
    pub(crate) handle_receipt: WorthUiRuntimeHandleAllocationReceipt,
    pub(crate) topology: WorthUiPlanTopology,
    pub(crate) lane_partitions: Vec<WorthUiPlanLanePartition>,
    pub(crate) lookup_index: WorthUiPlanLookupIndex,
    pub(crate) region_store: super::WorthUiPlanRegionStore,
    pub(crate) construction_counters: super::WorthUiPlanConstructionCounters,
    pub(crate) regional_evidence: super::WorthUiPlanRegionalEvidence,
    pub(crate) counters: WorthUiPlanTopologyCounters,
}

impl WorthUiExecutionPlan {
    pub(crate) fn new(
        authority: &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
        construction: WorthUiExecutionPlanConstruction,
    ) -> Self {
        let WorthUiExecutionPlanConstruction {
            handle_receipt,
            topology,
            lane_partitions,
            lookup_index,
            region_store,
            construction_counters,
            regional_evidence,
            counters,
        } = construction;
        Self {
            lowering_identity: authority.identity().clone(),
            handle_receipt,
            flat_projection: Some(super::WorthUiPlanFlatProjection::new(
                topology,
                lane_partitions,
                lookup_index,
            )),
            region_store,
            construction_counters,
            regional_evidence,
            counters,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_parts(
        &self,
        topology: WorthUiPlanTopology,
        lane_partitions: Vec<WorthUiPlanLanePartition>,
        lookup_index: WorthUiPlanLookupIndex,
        counters: WorthUiPlanTopologyCounters,
    ) -> Self {
        Self {
            lowering_identity: self.lowering_identity.clone(),
            handle_receipt: self.handle_receipt,
            flat_projection: Some(super::WorthUiPlanFlatProjection::new(
                topology,
                lane_partitions,
                lookup_index,
            )),
            region_store: self.region_store.clone(),
            construction_counters: self.construction_counters,
            regional_evidence: self.regional_evidence.clone(),
            counters,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_first_regional_family(
        &self,
        family: crate::runtime::WorthUiPlanNodeInputFamily,
    ) -> Self {
        let identity = self
            .region_store
            .canonical_identities()
            .into_iter()
            .find(|identity| {
                self.region_store
                    .schema_for(identity)
                    .is_some_and(|schema| schema.input().family() != family)
            })
            .expect("test plan has a region in another family");
        let input = self
            .region_store
            .schema_for(&identity)
            .expect("selected test region remains present")
            .input()
            .clone()
            .with_family_for_test(family);
        let successor =
            self.region_store
                .successor(vec![super::WorthUiPlanRegionMutation::Replace(
                    super::WorthUiPlanRegionSchema::from_node_input(input),
                )]);
        let mut changed = self.clone();
        changed.region_store = successor.into_store();
        changed
    }

    pub(crate) fn new_regional_successor(
        authority: &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
        handle_receipt: WorthUiRuntimeHandleAllocationReceipt,
        region_store: super::WorthUiPlanRegionStore,
        construction_counters: super::WorthUiPlanConstructionCounters,
        regional_evidence: super::WorthUiPlanRegionalEvidence,
        counters: WorthUiPlanTopologyCounters,
    ) -> Self {
        Self {
            lowering_identity: authority.identity().clone(),
            handle_receipt,
            flat_projection: None,
            region_store,
            construction_counters,
            regional_evidence,
            counters,
        }
    }

    pub(crate) fn shares_lowering_authority_with(
        &self,
        authority: &crate::runtime::planning::WorthUiExecutionPlanLoweringFacts,
    ) -> bool {
        self.lowering_identity
            .shares_authority_with(authority.identity())
    }

    pub(crate) fn shares_lowering_identity_with(
        &self,
        identity: &crate::runtime::planning::WorthUiExecutionPlanLoweringIdentity,
    ) -> bool {
        self.lowering_identity.shares_authority_with(identity)
    }

    pub fn handle_receipt(&self) -> WorthUiRuntimeHandleAllocationReceipt {
        self.handle_receipt
    }

    pub fn topology(&self) -> &WorthUiPlanTopology {
        self.flat_projection
            .as_ref()
            .expect("flat topology is reconstructive and was not materialized for this plan")
            .topology()
    }

    pub fn lane_partitions(&self) -> &[WorthUiPlanLanePartition] {
        self.flat_projection
            .as_ref()
            .expect("flat lane partitions are reconstructive and were not materialized")
            .lane_partitions()
    }

    pub fn lookup_index(&self) -> &WorthUiPlanLookupIndex {
        self.flat_projection
            .as_ref()
            .expect("flat lookup indexes are reconstructive and were not materialized")
            .lookup_index()
    }

    pub fn counters(&self) -> WorthUiPlanTopologyCounters {
        self.counters
    }

    pub fn region_count(&self) -> usize {
        self.region_store.region_count()
    }

    pub fn region_storage_counters(&self) -> super::WorthUiPlanRegionStorageCounters {
        self.construction_counters.regional_storage()
    }

    pub fn construction_counters(&self) -> super::WorthUiPlanConstructionCounters {
        self.construction_counters
    }

    pub fn regional_evidence(&self) -> &super::WorthUiPlanRegionalEvidence {
        &self.regional_evidence
    }

    pub fn canonical_region_identities(&self) -> Vec<super::WorthUiPlanRegionIdentity> {
        self.region_store.canonical_identities()
    }

    pub(crate) fn exactly_matches_executable_regions(
        &self,
        other: &Self,
    ) -> (bool, super::WorthUiPlanRegionStorageCounters) {
        self.region_store.exactly_matches(&other.region_store)
    }

    pub(crate) fn semantically_matches_executable_regions(
        &self,
        other: &Self,
    ) -> (bool, super::WorthUiPlanRegionStorageCounters) {
        self.region_store.semantically_matches(&other.region_store)
    }

    pub(crate) fn region_store(&self) -> &super::WorthUiPlanRegionStore {
        &self.region_store
    }

    pub(crate) fn regional_family_count(
        &self,
        family: crate::runtime::WorthUiPlanNodeInputFamily,
    ) -> usize {
        self.region_store.family_count(family)
    }

    pub(crate) fn mounted_projection_plan_index(&self, provenance: u64) -> Result<Option<u32>, ()> {
        self.region_store.mounted_projection_plan_index(provenance)
    }

    pub(crate) fn mounted_projection_ordinary_meaning(
        &self,
        plan_index: u32,
    ) -> Option<
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    > {
        self.region_store
            .mounted_projection_ordinary_meaning(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_with_identity(
        &self,
        plan_index: u32,
    ) -> Option<(
        String,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.region_store
            .mounted_projection_ordinary_meaning_with_identity(plan_index)
    }

    pub(crate) fn mounted_projection_ordinary_meaning_for_identity(
        &self,
        identity: &str,
    ) -> Option<(
        u32,
        std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        self.region_store
            .mounted_projection_ordinary_meaning_for_identity(identity)
    }

    pub(crate) fn mounted_projection_theme_token(
        &self,
        token_id: &crate::capability::ThemeTokenId,
    ) -> Result<
        Option<(
            u32,
            std::rc::Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
        )>,
        (),
    > {
        self.region_store.mounted_projection_theme_token(token_id)
    }

    pub(crate) fn regional_family_semantic_digest(
        &self,
        family: crate::runtime::WorthUiPlanNodeInputFamily,
    ) -> u64 {
        self.region_store.family_semantic_digest(family)
    }

    pub(crate) fn regional_family_slot_view<const N: usize>(
        &self,
        families: [crate::runtime::WorthUiPlanNodeInputFamily; N],
    ) -> super::WorthUiPlanRegionSlotSetView<N> {
        self.region_store.family_slot_view(families)
    }

    pub(crate) fn regional_root_shell_slot_view(&self) -> super::WorthUiPlanRegionSlotSetView<1> {
        self.region_store.root_shell_slot_view()
    }

    pub(crate) fn regional_store_clone(&self) -> super::WorthUiPlanRegionStore {
        self.region_store.clone()
    }

    pub(crate) fn regional_semantic_digest(&self) -> u64 {
        self.region_store.semantic_digest()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn reconstructive_inspection_rows(
        &self,
    ) -> Vec<(
        crate::runtime::WorthUiPlanNode,
        crate::runtime::WorthUiPlanNodeInput,
        crate::runtime::WorthUiPlanExecutionLane,
    )> {
        self.region_store
            .reconstructive_inspection_rows(self.handle_receipt.arena_identity())
    }

    pub(crate) fn has_reconstructive_flat_projection(&self) -> bool {
        self.flat_projection.is_some()
    }

    #[cfg(test)]
    pub(crate) fn shares_exact_region_storage_with(
        &self,
        other: &Self,
        identity: &super::WorthUiPlanRegionIdentity,
    ) -> bool {
        self.region_store
            .shares_exact_region_storage_with(&other.region_store, identity)
    }

    #[cfg(test)]
    pub(crate) fn region_storage_reclamation_probe_for_test(
        &self,
    ) -> super::WorthUiPlanRegionStorageReclamationProbe {
        self.region_store
            .reclamation_probe(!self.has_reconstructive_flat_projection())
    }
}

impl PartialEq for WorthUiExecutionPlan {
    fn eq(&self, other: &Self) -> bool {
        self.handle_receipt == other.handle_receipt
            && self.flat_projection == other.flat_projection
            && self.counters == other.counters
            && self.exactly_matches_executable_regions(other).0
    }
}

impl Eq for WorthUiExecutionPlan {}
