use worth_store::physical_runtime::{
    RecoveredMediaOwnedPhysicalRuntime, RecoveredPhysicalServingHandoff,
};

fn handoff_cannot_detach_lease(handoff: RecoveredPhysicalServingHandoff) {
    let RecoveredPhysicalServingHandoff { media, custody } = handoff;
    drop(media);
    let _custody_without_lease = custody;
}

fn admitted_owner_cannot_detach_lease(admitted: RecoveredMediaOwnedPhysicalRuntime) {
    let RecoveredMediaOwnedPhysicalRuntime { media, custody } = admitted;
    drop(media);
    let _custody_without_lease = custody;
}

fn main() {}
