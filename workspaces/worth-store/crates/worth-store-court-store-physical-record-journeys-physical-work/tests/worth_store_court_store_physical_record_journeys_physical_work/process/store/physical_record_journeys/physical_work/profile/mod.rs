#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::profile::*;
use {tempfile::tempdir};
use {worth_foundational::aspects, worth_foundational::AspectMask, worth_foundational::MutationMask, worth_foundational::ProjectionMask, worth_foundational::ScalarAspectType};
use {worth_signal::facade::PartitionSubscription};
use {worth_store::aspect_native::StoreAspectContractAdmission, worth_store::aspect_native::StoreAspectIdentity};
use {worth_store::physical_runtime::PhysicalSignalAspectBindingSet, worth_store::physical_runtime::PhysicalSignalAspectDeclaration, worth_store::physical_runtime::PhysicalSignalAspectRole, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkProfileDeclaration, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::PhysicalWorkSignalFamilySet};

use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::alternative_physical_witness, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope_from_authority, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT};

#[test]
fn inconsistent_capacity_profiles_have_no_construction_path() {
    assert!(worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 2, 1, 1, 1).is_none());
    assert!(worth_store::physical_runtime::PhysicalWorkCapacity::new(1, 1, 1, 2, 1).is_none());
}

#[test]
fn duplicate_profile_contracts_are_rejected_before_signal_construction() {
    let (_, _, admission, witness) = admitted_contract(1);
    assert!(matches!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(witness),
            [admission.clone(), admission]
        ),
        Err(worth_store::physical_runtime::PhysicalWorkProfileDenial::DuplicateAspectContract)
    ));
}

#[test]
fn profile_identity_is_canonical_across_contract_input_order() {
    let (_, _, first, witness) = admitted_contract(1);
    let second = contract_admission(
        "store.physical.work.secondary",
        72,
        worth_foundational::ScalarAspectType::String,
        witness,
    );
    let security = security_scope(witness);
    let forward =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security, [first.clone(), second.clone()]).unwrap();
    let reversed = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security, [second, first]).unwrap();
    assert_eq!(forward.identity(), reversed.identity());
}

#[test]
fn profile_identity_includes_the_admitted_security_authority() {
    let (_, _, admission, witness) = admitted_contract(1);
    let local =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), [admission.clone()]).unwrap();
    let foreign = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
        security_scope_from_authority("store.physical.foreign_profile_authority", witness),
        [admission],
    )
    .unwrap();

    assert_ne!(local.identity(), foreign.identity());
}

#[test]
fn binding_role_and_partition_are_canonical_profile_truth() {
    let (_, _, admission, witness) = admitted_contract(1);
    let partitioned_admission = admission
        .clone()
        .admit_mutation_mask(AspectMask::<MutationMask>::whole_aspect())
        .unwrap();
    let security = security_scope(witness);
    let dependency = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        security,
        [worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
            admission.clone(),
            worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
        )],
    )
    .unwrap();
    let partitioned = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        security,
        [worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
            partitioned_admission,
            worth_store::physical_runtime::PhysicalSignalAspectRole::DependencyAndOutput,
        )
        .with_partition(worth_signal::facade::PartitionSubscription::whole_partition("artifact-7"))],
    )
    .unwrap();
    assert_ne!(dependency.identity(), partitioned.identity());

    let dependency = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(dependency);
    let partitioned = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(partitioned);
    let left = dependency.binding_for_slot(0).unwrap();
    let right = partitioned.binding_for_slot(0).unwrap();
    assert_ne!(left.digest(), right.digest());
    assert!(left.partition().is_none());
    assert!(right.partition().is_some());
    assert_eq!(
        dependency
            .binding_for_identity(left.identity())
            .unwrap()
            .digest(),
        left.digest()
    );
}

#[test]
fn work_family_scope_is_canonical_and_empty_scope_is_rejected() {
    let (_, _, admission, witness) = admitted_contract(1);
    let security = security_scope(witness);
    let all = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        security,
        [worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
            admission.clone(),
            worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
        )],
    )
    .unwrap();
    let read_only = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        security,
        [worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
            admission.clone(),
            worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
        )
        .for_families(worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
            worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault,
        ))],
    )
    .unwrap();
    assert_ne!(all.identity(), read_only.identity());
    assert_ne!(
        worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(all)
            .binding_for_slot(0)
            .unwrap()
            .digest(),
        worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(read_only)
            .binding_for_slot(0)
            .unwrap()
            .digest()
    );
    assert!(matches!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security,
            [
                worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(
                    admission,
                    worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency,
                )
                .for_families(worth_store::physical_runtime::PhysicalWorkSignalFamilySet::none())
            ],
        ),
        Err(worth_store::physical_runtime::PhysicalWorkProfileDenial::WorkFamilySetEmpty)
    ));
}

#[test]
fn partitioned_dependency_profile_initializes_the_real_signal_owner() {
    let root = tempdir().unwrap();
    let (_, identity, admission, witness) = admitted_contract(1);
    let profile = worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
        security_scope(witness),
        [
            worth_store::physical_runtime::PhysicalSignalAspectDeclaration::new(admission, worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency)
                .with_partition(worth_signal::facade::PartitionSubscription::whole_partition("artifact-7")),
        ],
    )
    .unwrap();
    let declared_binding_count = u16::try_from(profile.contract_count()).unwrap();
    let route = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(profile.clone())
        .binding_for_identity(&identity)
        .unwrap()
        .digest();

    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let observation = serving.physical_signal_observation().unwrap();
    let installed_binding_count = declared_binding_count + EXPECTED_NATIVE_RECORD_BINDING_COUNT;
    assert_eq!(observation.aspect_binding_count(), installed_binding_count);
    assert_eq!(observation.locality_owner_count(), installed_binding_count);
    assert_eq!(
        serving.certification_physical_signal_route_depth(route),
        Some(0),
        "partitioned caller binding must retain its route through native installation"
    );
    serving.close();
}

#[test]
fn profile_identity_changes_with_exact_contract_shape_and_physical_witness() {
    let (_, _, string, witness) = admitted_contract(1);
    let integer = contract_admission(
        "store.physical.work.lifecycle",
        71,
        worth_foundational::ScalarAspectType::UInt64,
        witness,
    );
    let alternate_witness = worth_store::aspect_native::StoreAspectContractAdmission::new(
        integer.identity().clone(),
        integer.contract().clone(),
        alternative_physical_witness(),
    )
    .unwrap()
    .admit_projection_mask(AspectMask::<ProjectionMask>::whole_aspect())
    .unwrap();
    assert_ne!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), [string])
            .unwrap()
            .identity(),
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), [integer])
            .unwrap()
            .identity()
    );
    assert_ne!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(
            security_scope(witness),
            [contract_admission(
                "store.physical.work.lifecycle",
                71,
                worth_foundational::ScalarAspectType::UInt64,
                witness,
            )],
        )
        .unwrap()
        .identity(),
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), [alternate_witness])
            .unwrap()
            .identity()
    );
}

#[test]
fn signal_aspect_width_is_rejected_before_worker_construction() {
    let (_, _, _, witness) = admitted_contract(1);
    let contracts = (0..=worth_signal::facade::MAX_ASPECTS)
        .map(|index| {
            contract_admission(
                &format!("store.physical.work.profile.{index}"),
                1_000 + index as u64,
                worth_foundational::ScalarAspectType::String,
                witness,
            )
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), contracts),
        Err(worth_store::physical_runtime::PhysicalWorkProfileDenial::SignalAspectCapacityExceeded)
    ));
}

#[test]
fn signal_aspect_capacity_denial_stops_consuming_the_profile_source() {
    let (_, _, admission, witness) = admitted_contract(1);
    let hostile_unbounded_source =
        std::iter::repeat(admission)
            .enumerate()
            .map(|(index, admission)| {
                assert!(
                    index <= worth_signal::facade::MAX_ASPECTS,
                    "profile construction consumed beyond its declared Signal capacity"
                );
                admission
            });

    assert!(matches!(
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security_scope(witness), hostile_unbounded_source,),
        Err(worth_store::physical_runtime::PhysicalWorkProfileDenial::SignalAspectCapacityExceeded)
    ));
}
