//! Quote actual copied rule backing and both coexisting governance trees.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::storage::tree_clone_bytes;
use std::{alloc::Layout, sync::atomic::AtomicUsize};
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;

pub(super) fn inspection_disclosure_backing_bytes(
    inspection: &WorthQueryInstalledApplicationInspectionContract,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<RetainedStorageDenial>> {
    let visits = u64::try_from(
        inspection
            .definition()
            .parts()
            .metadata_disclosure
            .rules()
            .len(),
    )
    .map_err(|_| MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCounterOverflow))?;
    checkpoint(context, visits).map_err(MapKernelFailure::Stop)?;
    backing_bytes(inspection).ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))
}

fn backing_bytes(inspection: &WorthQueryInstalledApplicationInspectionContract) -> Option<u64> {
    let contract = &inspection.definition().parts().metadata_disclosure;
    if contract.posture() == ApplicationQueryDisclosurePosture::Public {
        return Some(0);
    }
    let strings = contract
        .classification()
        .len()
        .checked_add(contract.capability_name()?.len())?
        .checked_add(contract.capability_type()?.len())?;
    let trees = tree_clone_bytes::<(ApplicationQueryResultSlotKey, AdmittedDisclosureRule)>(
        contract.rules().len(),
    )?
    .checked_mul(2)?;
    let mut bytes = u64::try_from(strings).ok()?.checked_add(trees)?;
    for rule in contract.rules() {
        let slot = slot_for_rule(inspection, rule);
        let rule_header = Layout::new::<[AtomicUsize; 2]>()
            .extend(Layout::new::<WorthQueryAdmittedApplicationDisclosureRule>())
            .ok()?
            .0
            .pad_to_align()
            .size();
        let copied = slot
            .retained_heap_bytes()?
            .checked_add(rule.disclosure_value().owned_allocation_capacity_bytes())?
            .checked_add(rule_header)?;
        let influences = tree_clone_bytes::<ApplicationQueryObservableInfluence>(
            rule.influence().permitted().len(),
        )?;
        bytes = bytes
            .checked_add(u64::try_from(copied).ok()?)?
            .checked_add(influences)?;
    }
    Some(bytes)
}
