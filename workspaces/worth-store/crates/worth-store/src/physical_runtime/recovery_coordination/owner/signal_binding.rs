//! Exact signal aspects and partition names bound to one Store recovery session.

use crate::physical_runtime::instance::PhysicalWorkSignalOwner;
use crate::physical_runtime::{
    PhysicalSignalAspectRole, PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
};
use worth_signal::facade::PartitionSubscription;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

pub(super) fn bindings_match(
    signal: &PhysicalWorkSignalOwner,
    store: StableStoreIdentity,
    session: [u8; 16],
) -> bool {
    let observations = signal.binding_observations();
    let expected = [
        (
            "store.physical.recovery.discovery-basis",
            PhysicalSignalAspectRole::Dependency,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::ReadFault),
            expected_partition(store, session, "discovery"),
        ),
        (
            "store.physical.recovery.redo-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::ExactWriteback)
                .with(PhysicalWorkSignalFamily::Publication),
            expected_partition(store, session, "redo"),
        ),
        (
            "store.physical.recovery.publication-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::RootPublication),
            expected_partition(store, session, "publication"),
        ),
        (
            "store.physical.recovery.cleanup-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::WalReclamation),
            expected_partition(store, session, "cleanup"),
        ),
        (
            "store.physical.recovery.checkpoint-residue-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::CheckpointCapture),
            expected_partition(store, session, "checkpoint-residue"),
        ),
    ];
    observations.len() == expected.len()
        && expected.iter().all(|(key, role, families, partition)| {
            observations.iter().any(|observation| {
                observation.identity().aspect_key().as_str() == *key
                    && observation.role() == *role
                    && observation.families() == *families
                    && observation.partition().is_some_and(|actual| {
                        *actual == PartitionSubscription::whole_partition(partition.as_str())
                    })
            })
        })
}

fn expected_partition(store: StableStoreIdentity, session: [u8; 16], stage: &str) -> String {
    let store = store
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let session = session
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("store.physical.recovery/{store}/{session}/{stage}")
}
