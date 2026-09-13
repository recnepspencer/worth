use worth_store_physical_format::RootSelectorRole;
use worth_store_recovery_physics::{
    select_current_previous_root, PhysicalRootSlotObservation, SelectedPhysicalRoot,
};

use crate::entry::{PhysicalRecoveryBlockKind, PhysicalRecoverySourceDenial};
use crate::orchestration::BootstrapDiscovery;

use super::{PhysicalRecoveryDiscoveryCounters, SelectionFailure};

pub(super) fn select(
    current: PhysicalRootSlotObservation,
    previous: PhysicalRootSlotObservation,
    bootstrap: BootstrapDiscovery,
    mut root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    counters: PhysicalRecoveryDiscoveryCounters,
) -> Result<(SelectedPhysicalRoot, Vec<PhysicalRecoverySourceDenial>), SelectionFailure> {
    root_protocol_denials.extend(root_slot_denials(&current, &previous));
    let mut source_denials = root_protocol_denials.clone();
    let (anchor, bootstrap_denial) = match bootstrap {
        BootstrapDiscovery::Admitted(catalog) => (Some(catalog), None),
        BootstrapDiscovery::Rejected(denial) => (None, Some(denial)),
        BootstrapDiscovery::NotRequired | BootstrapDiscovery::Absent => (None, None),
    };
    select_current_previous_root(current, previous, anchor)
        .map(|selected| (selected, root_protocol_denials))
        .map_err(|denial| {
            if let Some(denial) = bootstrap_denial {
                source_denials.push(PhysicalRecoverySourceDenial::RootProtocol {
                    artifact: crate::entry::PhysicalRecoveryRootProtocolArtifact::BootstrapCatalog,
                    denial: denial.diagnostic(),
                });
            }
            source_denials.push(PhysicalRecoverySourceDenial::RootSelection(denial));
            SelectionFailure::new(
                PhysicalRecoveryBlockKind::RootProtocol,
                counters,
                "records/root selectors",
            )
            .with_source_denials(source_denials)
        })
}

fn root_slot_denials(
    current: &PhysicalRootSlotObservation,
    previous: &PhysicalRootSlotObservation,
) -> Vec<PhysicalRecoverySourceDenial> {
    [
        (RootSelectorRole::Current, current),
        (RootSelectorRole::Previous, previous),
    ]
    .into_iter()
    .filter_map(|(slot, observation)| {
        observation.rejection().map(
            |(denial, selector)| PhysicalRecoverySourceDenial::RootSlot {
                slot,
                denial,
                observed_store: selector.map(|value| value.store_identity()),
                observed_role: selector.map(|value| value.role()),
                observed_generation: selector.map(|value| value.root_generation()),
            },
        )
    })
    .collect()
}
