#[cfg(all())]
use {std::num::NonZeroU64};

#[cfg(all())]
use {worth_store_physical_format::RootSelectorRole};
use {worth_store_physical_format::BootstrapCatalog, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::RecordArtifactFile};
use {worth_store_recovery_runtime::PhysicalRecoveryOutcome};
#[cfg(all())]
use {worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority};
#[cfg(all())]
use {worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration};
use {worth_store_test_support::harness::physical_residency::canonical_physical_mutation_acknowledgment, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

#[cfg(all())]
fn selected_with_atomic_replacement_fault(
    root: &std::path::Path,
    replacement: u64,
) -> (
    worth_store_recovery_runtime::SelectedPhysicalRecovery,
    worth_store::physical_runtime::certification::CertificationMediaFaultActivation,
) {
    use {worth_store::physical_runtime::certification::MediaFaultDirective, worth_store::physical_runtime::certification::MediaOperationRole};

    selected_with_publication_fault(
        root,
        MediaOperationRole::AtomicReplace,
        replacement,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    )
}

#[cfg(all())]
pub(super) fn selected_with_publication_fault(
    root: &std::path::Path,
    role: worth_store::physical_runtime::certification::MediaOperationRole,
    occurrence: u64,
    directive: worth_store::physical_runtime::certification::MediaFaultDirective,
) -> (
    worth_store_recovery_runtime::SelectedPhysicalRecovery,
    worth_store::physical_runtime::certification::CertificationMediaFaultActivation,
) {
    use {worth_store::physical_runtime::FilesystemAccessPosture, worth_store::physical_runtime::FilesystemMediaAdmission};

    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let faults = admission.fault_schedule_authority();
    let activation = faults.one_shot_activation();
    let rule = faults
        .rule(role, occurrence, directive)
        .for_nth_identified_operation_after_activation(
            activation.clone(),
            std::num::NonZeroU64::new(occurrence).unwrap(),
        );
    let schedule = faults.schedule(vec![rule]).unwrap();
    let limits = crate::phase_four_planning::ordinary_recovery_limits(4_096);
    let configuration = worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire_for_certification(
        root.to_path_buf(),
        configuration.clone(),
        limits,
        schedule,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    let admitted = worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
    .admit()
    .unwrap();
    (admitted.discover().unwrap().select().unwrap(), activation)
}

#[cfg(all())]
fn assert_partial_root_protocol(replacement: u64, expected_prefix: usize) {
    let retained_root =
        crate::phase_four_planning::prepare_ordinary_recovery_root(&format!("c8-phase6-prefix-{replacement}"));
    let (selected, activation) =
        selected_with_atomic_replacement_fault(retained_root.path(), replacement);
    let planned = selected.plan().unwrap();
    let expected_candidate_count = planned.publication_plan().candidates().len() as u64;
    let candidates = planned
        .publication_plan()
        .candidates()
        .iter()
        .filter_map(|candidate| {
            let destination = match candidate.artifact() {
                worth_store_physical_format::RecordArtifactFile::RootSelectorCandidate {
                    role: worth_store_physical_format::RootSelectorRole::Previous,
                    ..
                } => worth_store_physical_format::RecordArtifactFile::PreviousRootSelector,
                worth_store_physical_format::RecordArtifactFile::RootSelectorCandidate {
                    role: worth_store_physical_format::RootSelectorRole::Current,
                    ..
                } => worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
                worth_store_physical_format::RecordArtifactFile::CatalogCandidate { .. } => worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                _ => return None,
            };
            Some((destination, candidate.bytes().to_vec()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let records = retained_root.path().join("families/records");
    let before = [
        worth_store_physical_format::RecordArtifactFile::PreviousRootSelector,
        worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
        worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
    ]
    .map(|artifact| {
        let bytes = std::fs::read(records.join(artifact.file_name())).ok();
        (artifact, bytes)
    });
    let staged = planned.stage().unwrap();
    activation.arm().unwrap();
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(outcome)) = staged.publish() else {
        panic!("partial root-protocol replacement must be publication-indeterminate")
    };
    assert!(matches!(
        outcome.settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::Indeterminate(
            worth_store::physical_runtime::PhysicalRecoveryPublicationCommandIndeterminate::Media {
                stage: worth_store::physical_runtime::PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement,
                ..
            }
        )
    ));
    assert_eq!(outcome.counters().namespace_synchronizations_performed, 0);
    assert_eq!(
        outcome.counters().candidate_artifacts_settled,
        expected_candidate_count
    );
    assert_eq!(
        outcome.counters().candidate_materializations_performed,
        expected_candidate_count
    );
    assert_eq!(
        outcome.counters().candidate_synchronizations_performed,
        expected_candidate_count
    );
    assert_eq!(outcome.counters().root_protocol_replacements_performed, 0);
    assert!(outcome.recovery_effects() > 0);
    for (index, (artifact, old)) in before.into_iter().enumerate() {
        let actual = std::fs::read(records.join(artifact.file_name())).unwrap();
        if index < expected_prefix {
            assert_eq!(actual, candidates[&artifact]);
        } else {
            assert_eq!(Some(actual), old);
        }
    }
}

#[test]
fn closed_staging_publishes_exact_candidates_before_the_namespace_barrier() {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root("c8-phase6-publication");
    let planned = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap();
    let expected_generation = planned.publication_plan().staging_generation();
    let expected_plan = planned.publication_plan().plan_identity();
    let expected_candidates = planned.publication_plan().candidates().len() as u64;
    let expected_effects = planned.publication_plan().expected_effects();

    let staged = planned.stage().unwrap();
    let published = staged.publish().unwrap();

    assert_eq!(
        published.publication_expectation().plan_identity(),
        expected_plan
    );
    assert_eq!(
        published
            .publication_expectation()
            .recovered_root()
            .generation(),
        expected_generation
    );
    let counters = published.publication_counters();
    assert_eq!(counters.planned_effects, expected_effects);
    assert_eq!(counters.candidate_artifacts_settled, expected_candidates);
    assert_eq!(
        counters.candidate_synchronizations_performed,
        expected_candidates
    );
    assert_eq!(counters.root_protocol_replacements_performed, 1);
    assert_eq!(counters.namespace_synchronizations_performed, 1);
    assert!(published.is_quiescent());

    let records = retained_root.path().join("families/records");
    let current = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .unwrap();
    let previous = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::PreviousRootSelector.file_name())).unwrap(),
    )
    .unwrap();
    let catalog = worth_store_physical_format::BootstrapCatalog::decode(
        &std::fs::read(records.join(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog.file_name())).unwrap(),
    )
    .unwrap();
    assert_eq!(current.root_generation(), expected_generation);
    assert_eq!(previous.root_generation(), expected_generation - 1);
    assert_eq!(current.linked_selector(), Some(previous.identity()));
    assert_eq!(
        current.linked_root_generation(),
        Some(previous.root_generation())
    );
    assert_eq!(previous.linked_selector(), Some(current.identity()));
    assert_eq!(
        previous.linked_root_generation(),
        Some(current.root_generation())
    );
    assert_eq!(
        catalog.current_root().generation().get(),
        expected_generation
    );
    assert_eq!(current.store_identity(), published.store_identity());

    let reopened = published.reopen().unwrap();
    assert_eq!(reopened.recovered_root().generation(), expected_generation);
    assert_eq!(
        reopened.recovered_root(),
        reopened.publication_expectation().recovered_root()
    );
    assert_eq!(reopened.reopen_counters().selector_reads_completed, 1);
    assert_eq!(reopened.reopen_counters().root_reads_completed, 1);
    assert!(reopened.reopen_counters().bytes_read > 0);
    assert!(reopened.is_quiescent());

    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("freshly reopened recovery must produce the recovered handoff");
    };
    assert_eq!(handoff.core().root().generation(), expected_generation);
    assert_eq!(handoff.core().store_identity(), current.store_identity());
    assert_eq!(
        handoff
            .selected_sources()
            .root()
            .selected()
            .selector()
            .root_generation(),
        expected_generation - 1
    );
    assert_eq!(
        handoff.freshness_sample().store_identity(),
        current.store_identity()
    );
    assert_eq!(
        handoff.closed_generation().generation(),
        expected_generation
    );
    assert_eq!(
        handoff.publication_expectation().recovered_root(),
        handoff.core().root()
    );
    assert_eq!(
        handoff
            .quiescence_plan()
            .expected_live_commands_after_close(),
        0
    );
    assert_eq!(
        handoff
            .quiescence_plan()
            .expected_live_media_handles_after_close(),
        0
    );
    assert_ne!(
        handoff.core().runtime_identity(),
        handoff.core().recovery_runtime_identity()
    );
    assert_eq!(handoff.reopen_counters().selector_reads_completed, 1);
    assert_eq!(handoff.reopen_counters().root_reads_completed, 1);
}

#[test]
fn an_already_durable_namespace_fresh_reopens_without_publication_effects() {
    use {worth_proof::TransitionOutcome};
    use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest};

    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("c8-phase6-clean").unwrap();
    let retained_root = world.retained_root();
    canonical_physical_mutation_acknowledgment(&world, [0x51; 32], b"already-durable");
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x52; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(5_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("clean checkpoint admission must succeed")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    drop(world);

    let published = crate::phase_four_planning::selected_ordinary_recovery(retained_root.path())
        .plan()
        .unwrap()
        .stage()
        .unwrap()
        .publish()
        .unwrap();
    assert_eq!(published.staging_counters().commands_submitted, 0);
    assert_eq!(published.staging_counters().commands_settled, 0);
    assert_eq!(published.publication_counters(), Default::default());
    assert!(published.is_quiescent());
    assert!(matches!(
        published.publication_settlement().settlement(),
        worth_store_recovery_runtime::PhysicalRecoveryPublicationSettlement::PreexistingNamespaceDurable
    ));
    assert!(published.reopen().is_ok());
}

#[cfg(all())]
#[test]
fn fault_after_previous_selector_retains_the_exact_one_of_three_prefix() {
    assert_partial_root_protocol(2, 1);
}

#[cfg(all())]
#[test]
fn fault_after_current_selector_retains_the_exact_two_of_three_prefix() {
    assert_partial_root_protocol(3, 2);
}
