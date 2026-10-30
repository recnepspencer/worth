//! Selected recovery fixtures armed at named publication fault boundaries.

#[cfg(all())]
fn selected_with_atomic_replacement_fault(
    root: &std::path::Path,
    replacement: u64,
) -> (
    worth_store_recovery_runtime::SelectedPhysicalRecovery,
    worth_store::physical_runtime::certification::CertificationMediaFaultActivation,
) {
    use {
        worth_store::physical_runtime::certification::MediaFaultDirective,
        worth_store::physical_runtime::certification::MediaOperationRole,
    };

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
pub(crate) fn selected_with_publication_fault(
    root: &std::path::Path,
    role: worth_store::physical_runtime::certification::MediaOperationRole,
    occurrence: u64,
    directive: worth_store::physical_runtime::certification::MediaFaultDirective,
) -> (
    worth_store_recovery_runtime::SelectedPhysicalRecovery,
    worth_store::physical_runtime::certification::CertificationMediaFaultActivation,
) {
    use {
        worth_store::physical_runtime::FilesystemAccessPosture,
        worth_store::physical_runtime::FilesystemMediaAdmission,
    };

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
    let configuration =
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority =
        worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire_for_certification(
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
pub(crate) fn assert_partial_root_protocol(replacement: u64, expected_prefix: usize) {
    let retained_root = crate::phase_four_planning::prepare_ordinary_recovery_root(&format!(
        "c8-phase6-prefix-{replacement}"
    ));
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
                worth_store_physical_format::RecordArtifactFile::CatalogCandidate { .. } => {
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog
                }
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
    let Err(worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        outcome,
    )) = staged.publish()
    else {
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
