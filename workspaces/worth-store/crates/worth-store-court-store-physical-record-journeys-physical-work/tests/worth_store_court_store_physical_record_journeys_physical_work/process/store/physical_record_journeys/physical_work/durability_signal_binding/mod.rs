use tempfile::tempdir;
use {
    worth_store::physical_runtime::PhysicalSignalAspectRole,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::PhysicalWorkSignalFamilySet,
};

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn durability_policy_installs_the_exact_dependency_signal_families() {
    let root = tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let before = serving.media_counters();

    let policy_binding = serving
        .physical_signal_aspect_binding_observations()
        .into_vec()
        .into_iter()
        .find(|binding| {
            binding.identity().aspect_key().as_str()
                == "store.physical.durability.policy-binding-basis"
        })
        .expect("the admitted durability policy must install one Signal binding");
    assert_eq!(
        policy_binding.role(),
        worth_store::physical_runtime::PhysicalSignalAspectRole::Dependency
    );
    assert_eq!(
        policy_binding.families(),
        worth_store::physical_runtime::PhysicalWorkSignalFamilySet::only(
            worth_store::physical_runtime::PhysicalWorkSignalFamily::WalAppend
        )
        .with(worth_store::physical_runtime::PhysicalWorkSignalFamily::DurabilityBarrier)
        .with(worth_store::physical_runtime::PhysicalWorkSignalFamily::CheckpointCapture)
        .with(worth_store::physical_runtime::PhysicalWorkSignalFamily::RootPublication)
        .with(worth_store::physical_runtime::PhysicalWorkSignalFamily::WalReclamation)
    );

    let policy_digest = serving
        .durability_observation()
        .policy_identity()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        policy_binding.partition().unwrap().path().segments()[0],
        format!("physical-durability-policy/{policy_digest}")
    );
    assert_eq!(serving.media_counters(), before);
    serving.close();
}
