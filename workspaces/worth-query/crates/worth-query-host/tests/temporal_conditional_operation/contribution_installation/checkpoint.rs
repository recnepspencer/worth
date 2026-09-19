use super::*;

pub(crate) fn application_checkpoint_restores_fresh_editable_authority() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let application = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: true,
        },),
        checkpoint_limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpointed", 0, 1, true);
            Ok(())
        },
    )
    .expect("the checkpoint source application installs");
    let original_world = application.current_world();
    let checkpoint = application
        .capture_application_checkpoint()
        .expect("the current committed world captures as opaque bytes");
    drop(application);

    let restored_contacts = ContactCounters::default();
    let restored_execution_probe = restored_contacts.clone();
    let (installation_predicate, _) = Predicate::controlled(restored_contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(restored_contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let restored = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts: restored_contacts,
            install_route: true,
        },),
        checkpoint_limits(),
        checkpoint,
    )
    .expect("the Query-issued application checkpoint restores");
    assert_eq!(
        restored_execution_probe.snapshot(),
        (0, 0, 0, 0),
        "restore must not execute conditional application work",
    );
    assert_ne!(restored.current_world(), original_world);
    let installed = restored
        .conditional::<TemporalConditional>()
        .expect("the restored contribution remains installed");
    let installed = installed
        .as_ref()
        .as_ref()
        .expect("the restored conditional route is live");
    change_input(
        &restored,
        &installed.invariant,
        restored.current_world(),
        "post-restore-edit",
    )
    .require_committed()
    .expect("fresh restored authority accepts a subsequent edit");
}

pub(crate) fn application_checkpoint_denies_corrupt_incompatible_and_forged_bytes() {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let application = application_installation::in_memory_program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: true,
        },),
        checkpoint_limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpoint-denial", 0, 1, true);
            Ok(())
        },
    )
    .expect("the denial courtroom source application installs");
    let checkpoint = application.capture_application_checkpoint().unwrap();
    let mut corrupt_payload = checkpoint.bytes().to_vec();
    *corrupt_payload.last_mut().unwrap() ^= 0x01;
    let forged_commit = checkpoint
        .rewrite_authenticated_body_for_durability_test(|body| {
            body[2..10].copy_from_slice(&u64::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();
    let unsupported_version = checkpoint
        .rewrite_authenticated_body_for_durability_test(|body| {
            body[..2].copy_from_slice(&u16::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();
    let invalid_payload_length = checkpoint
        .rewrite_authenticated_body_for_durability_test(|body| {
            body[10..18].copy_from_slice(&u64::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();

    for invalid in [
        Vec::new(),
        corrupt_payload,
        forged_commit,
        unsupported_version,
        invalid_payload_length,
    ] {
        checkpoint_bytes_are_denied(invalid);
    }
}

fn checkpoint_bytes_are_denied(corrupted_bytes: Vec<u8>) {
    let corrupt = application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
        corrupted_bytes.into_boxed_slice(),
    );
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let denial = match application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        (TemporalContributionConfiguration {
            installation_predicate,
            definition_predicate: Arc::new(definition_predicate),
            clock_source,
            clock_control,
            contacts,
            install_route: true,
        },),
        checkpoint_limits(),
        corrupt,
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("corrupt checkpoint bytes cannot publish an application"),
    };
    let application_installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denial
    else {
        panic!("corrupt checkpoint must be denied during graph recovery: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
    );
}

fn checkpoint_limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        product_world_resources(1_024),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
            .unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .unwrap(),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
