use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

pub(crate) fn application_checkpoint_restores_fresh_editable_authority(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint_limits(),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpointed", 0, 1, true);
            Ok(())
        },
    )
    .expect("the checkpoint source application installs");
    assert_eq!(application.checkpoint_restore_work(), None);
    let original_world = application.current_world();
    assert_live_relation_is_readable(&application, "source");
    let (checkpoint, sections) = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .expect("the current committed world captures as opaque bytes")
        })
        .expect("installed host call opens");
    assert_eq!(sections.total_bytes(), checkpoint.bytes().len());
    assert!(sections.native_bytes() > 0);
    assert_eq!(
        sections.accepted_output_count() == 0,
        sections.accepted_output_bytes() == 0
    );
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint_limits(),
        checkpoint,
    )
    .expect("the Query-issued application checkpoint restores");
    let restore_work = restored
        .checkpoint_restore_work()
        .expect("native readmission reports its actual restore work");
    assert_eq!(
        restore_work.native_bytes_read,
        Some(sections.native_bytes())
    );
    assert!(restore_work.native_envelopes_readmitted.unwrap_or_default() > 0);
    assert!(restore_work.root_images_verified > 0);
    assert!(restore_work.branch_cells_readmitted > 0);
    assert_eq!(
        restored_execution_probe.snapshot(),
        (0, 0, 0, 0),
        "restore must not execute conditional application work",
    );
    assert_ne!(restored.current_world(), original_world);
    assert_live_relation_is_readable(&restored, "restored");
    let installed = restored
        .conditional::<TemporalConditional>()
        .expect("the restored contribution remains installed");
    let installed = installed
        .as_ref()
        .as_ref()
        .expect("the restored conditional route is live");
    change_input(
        request_execution,
        &restored,
        &installed.invariant,
        restored.current_world(),
        "post-restore-edit",
    )
    .require_committed()
    .expect("fresh restored authority accepts a subsequent edit");
}

fn assert_live_relation_is_readable(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    phase: &str,
) {
    let request = request_scope();
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(application.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let principal = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &principal_binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let scope = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let query = schema
        .certification_query(TemporalIntentLiveQuery::reference())
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = application.with_host_advancement(|host_phase| selected
        .admit_application_query(
            &host_phase, &query,
            &access,
            worth_query_host::facade::declaration::application_query::ApplicationQueryParameterSet::new(),
            primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(8).unwrap(), &request),
        )
        .unwrap()).expect("installed host call opens");
    let result = application
        .with_host_advancement(|phase| {
            application
                .execute_application_query_one_shot(&phase, plan)
                .expect("checkpointed relation remains readable")
        })
        .expect("installed host call opens");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].targets, vec!["intent-1"], "{phase}");
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint_limits(),
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpoint-denial", 0, 1, true);
            Ok(())
        },
    )
    .expect("the denial courtroom source application installs");
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    let mut corrupt_payload = checkpoint.bytes().to_vec();
    *corrupt_payload.last_mut().unwrap() ^= 0x01;
    // Independent current grammar: format, workflow facts, workflow instances,
    // then bootstrap u64 and native payload length u64.
    const BOOTSTRAP: usize = 3 * std::mem::size_of::<u16>();
    const NATIVE_LENGTH: usize = BOOTSTRAP + std::mem::size_of::<u64>();
    let forged_commit = checkpoint
        .rewrite_authenticated_checkpoint_body_for_test(|body| {
            body[BOOTSTRAP..BOOTSTRAP + 8].copy_from_slice(&u64::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();
    let unsupported_version = checkpoint
        .rewrite_authenticated_checkpoint_body_for_test(|body| {
            body[..2].copy_from_slice(&u16::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();
    let unsupported_workflow = checkpoint
        .rewrite_authenticated_checkpoint_body_for_test(|body| {
            body[2..4].copy_from_slice(&u16::MAX.to_be_bytes());
            body[4..6].copy_from_slice(&u16::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();
    let invalid_payload_length = checkpoint
        .rewrite_authenticated_checkpoint_body_for_test(|body| {
            body[NATIVE_LENGTH..NATIVE_LENGTH + 8].copy_from_slice(&u64::MAX.to_be_bytes());
        })
        .bytes()
        .to_vec();

    use primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
    for (invalid, expected, cause) in [
        (
            Vec::new(),
            Kind::CheckpointRecoveryRejected,
            "header is invalid",
        ),
        (
            corrupt_payload,
            Kind::CheckpointRecoveryRejected,
            "checksum differs",
        ),
        (
            forged_commit,
            Kind::CheckpointRecoveryRejected,
            "omitted its claimed bootstrap commit",
        ),
        (
            unsupported_version,
            Kind::UnsupportedCheckpointFormat {
                observed: u16::MAX,
                supported: 10,
            },
            "format 65535 is unsupported",
        ),
        (
            unsupported_workflow,
            Kind::UnsupportedCheckpointWorkflowContract {
                facts: u16::MAX,
                instances: u16::MAX,
            },
            "workflow contract is unsupported",
        ),
        (
            invalid_payload_length,
            Kind::CheckpointRecoveryRejected,
            "checkpoint payload",
        ),
    ] {
        checkpoint_bytes_are_denied(invalid, expected, cause);
    }
}

fn checkpoint_bytes_are_denied(
    corrupted_bytes: Vec<u8>,
    expected: primary_graph::WorthQueryPrimaryGraphInstallationDenialKind,
    cause: &str,
) {
    let corrupt = application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
        corrupted_bytes.into_boxed_slice(),
    );
    let contacts = ContactCounters::default();
    let observed_contacts = contacts.clone();
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
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
    assert_eq!(denial.kind(), expected, "{denial:?}");
    assert!(denial.subject().contains(cause), "{denial:?}");
    assert_eq!(observed_contacts.snapshot(), (0, 0, 0, 0));
}

pub(super) fn checkpoint_limits() -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        product_world_resources(1_024),
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
        ),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
