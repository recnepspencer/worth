use super::BridgeSemanticClaimOrigin;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::RwLockWriteGuard;

use worth_proof::TransitionOutcome;
use worth_signal::facade::{InstalledSignalAspectSetCapability, SignalGraph};

use crate::facade::RuntimeBridge;

use super::admission::CorrespondenceAdmissionOutcome;
use super::slot_allocation::{AllocationKey, CorrespondenceAllocationRegistry};
use super::target_allocation::PlannedCorrespondence;
use super::{
    BridgeCorrespondenceAdmissionFailure, BridgeCorrespondenceDeferred,
    BridgeCorrespondenceDenialKind, BridgeInstalledSemanticCorrespondence,
    BridgeSemanticDependencyCandidate,
};

pub(crate) fn isolate_allocation_state(
    runtime: &mut RuntimeBridge,
) -> Result<CorrespondenceAllocationRegistry, BridgeCorrespondenceAdmissionFailure> {
    let retained = runtime
        .correspondence_allocations
        .read()
        .map_err(|_| BridgeCorrespondenceAdmissionFailure::LockPoisoned)?
        .clone();
    runtime.correspondence_allocations =
        std::sync::Arc::new(std::sync::RwLock::new(retained.clone()));
    Ok(retained)
}

/// A mutation-free correspondence batch whose allocation lock is retained
/// until its caller has admitted every other authority participating in the
/// installation. Dropping it rolls the whole batch back.
pub(crate) struct PreparedCorrespondenceBatch<'runtime> {
    registry: RwLockWriteGuard<'runtime, CorrespondenceAllocationRegistry>,
    items: Vec<(PlannedCorrespondence, InstalledSignalAspectSetCapability)>,
}

impl PreparedCorrespondenceBatch<'_> {
    pub(crate) fn dependency_aspects(&self) -> worth_signal::facade::AspectMask {
        self.aspect_mask_for(|_| true)
    }

    pub(crate) fn condition_aspects(
        &self,
        dependency_ordinals: &[usize],
    ) -> worth_signal::facade::AspectMask {
        self.aspect_mask_for(|candidate| {
            dependency_ordinals.contains(&candidate.dependency_ordinal())
        })
    }

    fn aspect_mask_for(
        &self,
        include: impl Fn(&BridgeSemanticDependencyCandidate) -> bool,
    ) -> worth_signal::facade::AspectMask {
        let mut mask = worth_signal::facade::AspectMask::EMPTY;
        for (planned, _) in &self.items {
            if include(planned.resolved.recipe.payload()) {
                for target in planned.targets.as_slice() {
                    mask.insert(target.aspect);
                }
            }
        }
        mask
    }

    pub(crate) fn commit(mut self) -> Vec<BridgeInstalledSemanticCorrespondence> {
        let mut installed = Vec::with_capacity(self.items.len());
        for (mut planned, signal_targets) in self.items {
            planned.resolved.counters.authoritative_records_committed =
                planned.pending_records.len();
            for record in planned.pending_records {
                self.registry.commit(record);
            }
            installed.push(BridgeInstalledSemanticCorrespondence::admit_ready(
                planned.resolved.recipe,
                planned.targets,
                planned.resolved.counters,
                &planned.resolved.signal_graph,
                &signal_targets,
            ));
        }
        installed
    }
}

pub(crate) fn prepare_registered_correspondence_batch<'runtime>(
    runtime: &'runtime RuntimeBridge,
    registrations: &[super::BridgeSemanticCorrespondenceRegistration],
    graph: &SignalGraph,
) -> Result<PreparedCorrespondenceBatch<'runtime>, CorrespondenceAdmissionOutcome> {
    let mut mapped = Vec::with_capacity(registrations.len());
    for registration in registrations {
        let resolved = super::resolution::resolve_registration(runtime, registration, graph)?;
        mapped.push(super::target_mapping::map_targets(runtime, resolved)?);
    }
    prepare_mapped_correspondence_batch(runtime, mapped, graph)
}

fn prepare_mapped_correspondence_batch<'runtime>(
    runtime: &'runtime RuntimeBridge,
    mut mapped: Vec<super::target_mapping::MappedCorrespondence>,
    graph: &SignalGraph,
) -> Result<PreparedCorrespondenceBatch<'runtime>, CorrespondenceAdmissionOutcome> {
    if let Some(first) = mapped.first_mut() {
        first.resolved.counters.allocation_registry_lock_attempts += 1;
    }
    let registry = match runtime.correspondence_allocations.try_write() {
        Ok(registry) => registry,
        Err(std::sync::TryLockError::WouldBlock) => {
            return Err(TransitionOutcome::Deferred(
                BridgeCorrespondenceDeferred::GraphMutationInProgress,
            ));
        }
        Err(std::sync::TryLockError::Poisoned(_)) => {
            return Err(TransitionOutcome::Failed(
                BridgeCorrespondenceAdmissionFailure::LockPoisoned,
            ));
        }
    };
    prepare_allocations(mapped, registry, graph)
}

fn prepare_allocations<'runtime>(
    mapped: Vec<super::target_mapping::MappedCorrespondence>,
    registry: RwLockWriteGuard<'runtime, CorrespondenceAllocationRegistry>,
    graph: &SignalGraph,
) -> Result<PreparedCorrespondenceBatch<'runtime>, CorrespondenceAdmissionOutcome> {
    let mut pending_owners = BTreeMap::<AllocationKey, BTreeSet<BridgeSemanticClaimOrigin>>::new();
    let mut items = Vec::with_capacity(mapped.len());
    for mapped in mapped {
        let mut planned =
            super::target_allocation::plan_with_registry(mapped, &registry, &mut pending_owners)?;
        let signal_targets = match graph.admit_installed_aspects(
            planned
                .targets
                .as_slice()
                .iter()
                .map(|target| (target.node, target.aspect)),
        ) {
            TransitionOutcome::Success(capability) => capability,
            _ => {
                return Err(super::admission::denied(
                    BridgeCorrespondenceDenialKind::MissingOrStaleSignalNode,
                    planned.resolved.counters,
                ));
            }
        };
        planned.resolved.counters.signal_node_admissions = signal_targets.aspects().len();
        planned.resolved.counters.targets_admitted = planned.targets.as_slice().len();
        items.push((planned, signal_targets));
    }
    Ok(PreparedCorrespondenceBatch { registry, items })
}
