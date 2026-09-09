use worth_proof::TransitionOutcome;
use worth_signal::facade::{CanonicalChangedRegions, InstalledSignalScopedChange};

use super::super::{BridgeDeliveredCorrespondenceChangeSet, BridgeInstalledSemanticCorrespondence};

#[derive(Debug, Clone)]
struct BridgePreparedScopedSignalTarget {
    node: worth_signal::facade::NodeId,
    aspect: worth_signal::facade::Aspect,
    changed_regions: CanonicalChangedRegions,
}

#[derive(Debug, Clone)]
pub struct BridgePreparedScopedSignalInvalidation {
    graph_instance_id: u64,
    targets: Vec<BridgePreparedScopedSignalTarget>,
}

impl BridgePreparedScopedSignalInvalidation {
    pub const fn graph_instance_id(&self) -> u64 {
        self.graph_instance_id
    }

    pub fn target_count(&self) -> usize {
        self.targets.len()
    }

    pub fn changed_regions(&self) -> impl ExactSizeIterator<Item = &CanonicalChangedRegions> {
        self.targets.iter().map(|target| &target.changed_regions)
    }

    pub(crate) fn retains_target(
        &self,
        graph_instance_id: u64,
        node: worth_signal::facade::NodeId,
        aspect: worth_signal::facade::Aspect,
    ) -> bool {
        self.graph_instance_id == graph_instance_id
            && self
                .targets
                .iter()
                .any(|target| target.node == node && target.aspect == aspect)
    }

    pub(crate) fn has_unique_target_bindings(&self) -> bool {
        let mut bindings = std::collections::BTreeSet::new();
        self.targets
            .iter()
            .all(|target| bindings.insert((target.node, target.aspect)))
    }

    pub(crate) fn signal_delivery_request(
        &self,
    ) -> worth_signal::facade::branch::SignalCommittedPatchDeliveryRequest {
        worth_signal::facade::branch::SignalCommittedPatchDeliveryRequest::new(
            self.targets.iter().map(|target| {
                worth_signal::facade::branch::SignalCommittedPatchTarget::new(
                    self.graph_instance_id,
                    target.node,
                    target.aspect,
                    target.changed_regions.as_slice().iter().cloned(),
                )
            }),
        )
    }

    pub(crate) fn admit_raw_graph(
        &self,
        graph: &mut worth_signal::facade::SignalGraph,
        counters: &mut super::super::CorrespondenceDeliveryCounters,
    ) -> Result<Vec<InstalledSignalScopedChange>, ()> {
        let mut changes = Vec::with_capacity(self.targets.len());
        for target in &self.targets {
            let TransitionOutcome::Success(capability) =
                graph.admit_installed_aspect(target.node, target.aspect)
            else {
                return Err(());
            };
            if capability.graph_instance_id() != self.graph_instance_id {
                return Err(());
            }
            changes.push(InstalledSignalScopedChange::new(
                capability,
                target.changed_regions.as_slice().iter().cloned(),
            ));
            counters.signal_capability_admissions += 1;
        }
        Ok(changes)
    }
}

pub(crate) fn prepare_scoped_signal_invalidation_for_targets(
    correspondence: &BridgeInstalledSemanticCorrespondence,
    targets: &[super::super::InstalledCorrespondenceTarget],
    change_set: &BridgeDeliveredCorrespondenceChangeSet,
) -> BridgePreparedScopedSignalInvalidation {
    let mut prepared_targets = Vec::with_capacity(targets.len());
    for target in targets {
        let regions = super::super::locality_lowering::lower_installed_target_regions(
            change_set.dependency(),
            target,
            change_set.changes(),
        );
        prepared_targets.push(BridgePreparedScopedSignalTarget {
            node: target.node,
            aspect: target.aspect,
            changed_regions: regions.clone(),
        });
    }
    BridgePreparedScopedSignalInvalidation {
        graph_instance_id: correspondence.basis().signal_graph_instance_id,
        targets: prepared_targets,
    }
}
