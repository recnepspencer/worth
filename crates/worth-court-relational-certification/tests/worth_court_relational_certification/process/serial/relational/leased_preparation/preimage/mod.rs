





#[allow(unused_imports)] use crate::leased_preparation::isolated_context::preimage::*;
pub(crate) use {worth_foundational::facade::AspectValue, worth_execution::CancellationToken, worth_relational::facade::identity::KindId, crate::leased_preparation::ordinary_context::authority, crate::leased_preparation::ordinary_context::lease_request, crate::leased_preparation::ordinary_context::serial};
pub(crate) use {crate::leased_preparation::isolated_context::preimage::source::Dependencies, crate::leased_preparation::isolated_context::preimage::source::Source};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::ContractValidatedAspectValueView, worth_foundational::facade::FieldKey};
pub(crate) use {worth_relational::facade::snapshots::SnapshotHandle};

#[test]
fn preimage_endpoint_partitions() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    for partitions in [8, 9] {
        journey(partitions, 1, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints);
    }
}

#[test]
fn preimage_read_partitions() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    for partitions in [8, 9] {
        journey(partitions, 1, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Reads);
    }
}

#[test]
fn preimage_endpoint_slots() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    for slots in [1024, 1025] {
        journey(1, slots, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Endpoints);
    }
}

#[test]
fn preimage_read_slots() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    for slots in [1024, 1025] {
        journey(1, slots, crate::leased_preparation::isolated_context::preimage::source::Dependencies::Reads);
    }
}
#[path = "creation/mod.rs"]
pub(crate) mod creation;
#[path = "refusal/mod.rs"]
pub(crate) mod refusal;
