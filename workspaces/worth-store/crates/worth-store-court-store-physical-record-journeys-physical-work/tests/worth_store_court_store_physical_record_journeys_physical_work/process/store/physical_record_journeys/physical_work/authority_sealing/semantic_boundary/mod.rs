use tempfile::tempdir;
use {worth_foundational::AspectMask, worth_foundational::MutationMask};
use {
    worth_store::aspect_native::StoreAspectContractAdmission,
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalWorkConcurrencyRelation,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkProfileDenial,
};

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::disjoint_artifact_mutation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn legacy_signal_resource_construction_is_forbidden() {
    let root = tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = crate::physical_record_journeys::physical_work::readiness::success(
        serving.physical_read_submission().submit(request.clone()),
    );
    let second = crate::physical_record_journeys::physical_work::readiness::success(
        serving.physical_read_submission().submit(request),
    );
    let first = serving.admit_physical_work(first).unwrap();
    let second = serving.admit_physical_work(second).unwrap();
    let first = serving.request_physical_work(first).unwrap();
    let second = serving.request_physical_work(second).unwrap_or_else(|denial| {
        panic!(
            "C5_PREDICATE:legacy-resource-node: local resource-node state overrode canonical Signal admission: {denial:?}"
        )
    });
    drop((first, second));
    serving.close();
}

#[test]
fn raw_signal_slots_cannot_become_semantic_authority() {
    let (_, _, first, first_witness) = admitted_named_contract("store.physical.slot.first", 901, 1);
    let (_, _, second, second_witness) =
        admitted_named_contract("store.physical.slot.second", 902, 1);
    let first = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(first_witness),
            [first],
        )
        .unwrap(),
    );
    let second = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(second_witness),
            [second],
        )
        .unwrap(),
    );

    assert_ne!(
        first.binding_for_slot(0).unwrap().digest(),
        second.binding_for_slot(0).unwrap().digest(),
        "C5_PREDICATE:raw-signal-slot-authority: raw slot equality replaced Store-native semantic identity"
    );
}

#[test]
fn foundational_masks_cannot_substitute_for_native_bindings() {
    let (contract, identity, _, witness) =
        admitted_named_contract("store.physical.mask.mutation-only", 903, 1);
    let mutation_only =
        worth_store::aspect_native::StoreAspectContractAdmission::new(identity, contract, witness)
            .unwrap()
            .admit_mutation_mask(AspectMask::<MutationMask>::whole_aspect())
            .unwrap();
    assert!(
        matches!(
            worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), [mutation_only]),
            Err(worth_store::physical_runtime::PhysicalWorkProfileDenial::DependencyProjectionMaskAbsent)
        ),
        "C5_PREDICATE:foundational-mask-substitution: Signal mask substituted for absent native projection authority"
    );
}

#[test]
fn callers_cannot_broaden_aspect_or_partition_scope() {
    let root = tempdir().unwrap();
    let (profile, first, second) = disjoint_artifact_mutation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let first = admitted_write(&serving, first);
    let second = admitted_write(&serving, second);

    assert_eq!(
        first
            .concurrency_scope()
            .relation(&second.concurrency_scope()),
        worth_store::physical_runtime::PhysicalWorkConcurrencyRelation::DisjointArtifacts,
        "C5_PREDICATE:aspect-partition-broadening: caller aspect or partition state broadened exact coordinate scope"
    );
    serving.close();
}
