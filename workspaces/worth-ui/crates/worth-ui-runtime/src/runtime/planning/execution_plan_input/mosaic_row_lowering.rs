use std::collections::BTreeMap;

use crate::runtime::{
    WorthUiNodeLifecycleTransition, WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily,
    WorthUiPlanNodeTopologyInput,
};
use crate::source::{
    WorthUiMosaicMountFacts, WorthUiMosaicRegionFacts, WorthUiMosaicStructureFacts,
};

use super::ordinary_lowering::WorthUiOrdinaryLoweringDenial;
use super::{
    durable_family_for_slot, WorthUiChildRangePlanMeaning, WorthUiLayoutPlanMeaning,
    WorthUiPlanOrdinaryMeaning, WorthUiStateSlotPlanMeaning, WorthUiStateSlotSuccession,
};

pub(super) struct WorthUiMosaicRowLowerer<'a> {
    root_identity: &'a str,
    provenance: Option<u64>,
    transition: WorthUiNodeLifecycleTransition,
    reconciliation: Option<&'a crate::runtime::WorthUiDurableStateReconciliationPlan>,
    rows: Vec<WorthUiPlanNodeInput>,
}

pub(super) struct WorthUiLoweredMosaicRows {
    pub(super) rows: Vec<WorthUiPlanNodeInput>,
    pub(super) root_children: Vec<String>,
}

impl<'a> WorthUiMosaicRowLowerer<'a> {
    pub(super) fn new(
        root_identity: &'a str,
        provenance: Option<u64>,
        transition: WorthUiNodeLifecycleTransition,
        reconciliation: Option<&'a crate::runtime::WorthUiDurableStateReconciliationPlan>,
    ) -> Self {
        Self {
            root_identity,
            provenance,
            transition,
            reconciliation,
            rows: Vec::new(),
        }
    }

    pub(super) fn lower(
        mut self,
        structure: &WorthUiMosaicStructureFacts,
    ) -> Result<WorthUiLoweredMosaicRows, WorthUiOrdinaryLoweringDenial> {
        let mut occurrences = BTreeMap::<&str, usize>::new();
        let mut root_children = Vec::with_capacity(structure.root_regions().len());
        for region in structure.root_regions() {
            let id = region.region().id().as_str();
            let occurrence = next_occurrence(&mut occurrences, id);
            let identity = child_identity(self.root_identity, "region", id, occurrence);
            self.lower_region(identity.clone(), region)?;
            root_children.push(identity);
        }
        Ok(WorthUiLoweredMosaicRows {
            rows: self.rows,
            root_children,
        })
    }

    fn lower_region(
        &mut self,
        identity: String,
        region: &WorthUiMosaicRegionFacts,
    ) -> Result<(), WorthUiOrdinaryLoweringDenial> {
        let mut children = Vec::new();
        if let Some((_, descriptor)) = region.state_slot() {
            children.push(self.lower_state_slot(&identity, descriptor)?);
        }
        let mut region_occurrences = BTreeMap::<&str, usize>::new();
        for child in region.child_regions() {
            let id = child.region().id().as_str();
            let occurrence = next_occurrence(&mut region_occurrences, id);
            let child_row_identity = child_identity(&identity, "region", id, occurrence);
            self.lower_region(child_row_identity.clone(), child)?;
            children.push(child_row_identity);
        }
        let mut mount_occurrences = BTreeMap::<&str, usize>::new();
        for mount in region.mounts() {
            let id = mount.surface().id().as_str();
            let occurrence = next_occurrence(&mut mount_occurrences, id);
            let mount_identity = child_identity(&identity, "mount", id, occurrence);
            self.lower_mount(mount_identity.clone(), mount)?;
            children.push(mount_identity);
        }
        let range_identity = self.lower_child_range(&identity, children);
        self.rows.push(WorthUiPlanNodeInput::from_ordinary_row(
            identity,
            self.provenance,
            WorthUiPlanNodeInputFamily::LayoutRegion,
            self.transition,
            WorthUiPlanNodeTopologyInput::empty(),
            Some(self.root_identity.to_owned()),
            WorthUiPlanOrdinaryMeaning::Layout(WorthUiLayoutPlanMeaning::region(
                region.descriptor().clone(),
                region
                    .sizing_contract()
                    .map(|(_, descriptor)| descriptor.clone()),
                range_identity,
            )),
        ));
        Ok(())
    }

    fn lower_mount(
        &mut self,
        identity: String,
        mount: &WorthUiMosaicMountFacts,
    ) -> Result<(), WorthUiOrdinaryLoweringDenial> {
        let mut children = Vec::new();
        if let Some((_, descriptor)) = mount.state_slot() {
            children.push(self.lower_state_slot(&identity, descriptor)?);
        }
        let range_identity = self.lower_child_range(&identity, children);
        self.rows.push(WorthUiPlanNodeInput::from_ordinary_row(
            identity,
            self.provenance,
            WorthUiPlanNodeInputFamily::LayoutRegion,
            self.transition,
            WorthUiPlanNodeTopologyInput::empty(),
            Some(self.root_identity.to_owned()),
            WorthUiPlanOrdinaryMeaning::Layout(WorthUiLayoutPlanMeaning::surface(
                mount.descriptor().clone(),
                mount
                    .placement_policy()
                    .map(|(_, descriptor)| descriptor.clone()),
                range_identity,
            )),
        ));
        Ok(())
    }

    fn lower_state_slot(
        &mut self,
        parent_identity: &str,
        descriptor: &crate::capability::MosaicStateSlotDescriptor,
    ) -> Result<String, WorthUiOrdinaryLoweringDenial> {
        let identity = child_identity(parent_identity, "state", descriptor.id().as_str(), 0);
        let succession = match self.reconciliation {
            None => WorthUiStateSlotSuccession::Launch,
            Some(plan) => WorthUiStateSlotSuccession::Reconciled(
                plan.receipt_for(self.root_identity, &durable_family_for_slot(descriptor))
                    .cloned()
                    .ok_or(WorthUiOrdinaryLoweringDenial::MissingStateSuccession)?,
            ),
        };
        let meaning = WorthUiStateSlotPlanMeaning::new(
            self.root_identity.to_owned(),
            descriptor.clone(),
            succession,
        )
        .map_err(|_| WorthUiOrdinaryLoweringDenial::InvalidStateSuccession)?;
        self.rows.push(WorthUiPlanNodeInput::from_ordinary_row(
            identity.clone(),
            self.provenance,
            WorthUiPlanNodeInputFamily::StateSlot,
            self.transition,
            WorthUiPlanNodeTopologyInput::empty(),
            Some(self.root_identity.to_owned()),
            WorthUiPlanOrdinaryMeaning::StateSlot(meaning),
        ));
        Ok(identity)
    }

    fn lower_child_range(&mut self, owner_identity: &str, children: Vec<String>) -> Option<String> {
        if children.is_empty() {
            return None;
        }
        let identity = format!("{owner_identity}::child-range");
        self.rows.push(WorthUiPlanNodeInput::from_ordinary_row(
            identity.clone(),
            self.provenance,
            WorthUiPlanNodeInputFamily::ChildRange,
            self.transition,
            WorthUiPlanNodeTopologyInput::empty(),
            Some(self.root_identity.to_owned()),
            WorthUiPlanOrdinaryMeaning::ChildRange(WorthUiChildRangePlanMeaning::new(
                owner_identity.to_owned(),
                children,
            )),
        ));
        Some(identity)
    }
}

fn child_identity(parent: &str, kind: &str, id: &str, occurrence: usize) -> String {
    format!("{parent}::{kind}::{id}#{occurrence}")
}

fn next_occurrence<'a>(occurrences: &mut BTreeMap<&'a str, usize>, id: &'a str) -> usize {
    let occurrence = occurrences.entry(id).or_default();
    let current = *occurrence;
    *occurrence += 1;
    current
}
