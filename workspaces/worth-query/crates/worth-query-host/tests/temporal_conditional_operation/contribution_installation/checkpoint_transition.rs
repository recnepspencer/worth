//! Real native predecessor -> target World -> ordinary target reopen journey.
use super::*;
use application_installation::{
    WorthQueryCheckpointProgramPredecessor as Predecessor,
    WorthQueryCheckpointTransitionResources as Resources,
};

struct LegacyTemporalProgram;
impl ApplicationProgramDefinition<TemporalHostSchema> for LegacyTemporalProgram {
    type Contributions = <TemporalHostSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.host.temporal-installation.legacy");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        TemporalInstallationProgram::feature_specs()
    }
}

fn configuration() -> (TemporalContributionConfiguration,) {
    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    (TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts,
        install_route: true,
    },)
}

fn source() -> (
    application_installation::WorthQueryApplicationCheckpoint,
    Predecessor,
) {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    let predecessor = Predecessor::new(&program.revision().to_string()).unwrap();
    let application = application_installation::in_memory_program(
        program,
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
        |graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "checkpoint-transition", 0, 1, true);
            Ok(())
        },
    )
    .unwrap();
    (
        application.capture_application_checkpoint().unwrap(),
        predecessor,
    )
}

fn transition(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    predecessor: Predecessor,
    resources: Resources,
    author: impl FnOnce(
        &mut application_installation::WorthQueryCheckpointMigrationWriter<'_, TemporalHostSchema>,
        &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    application_installation::WorthQueryInMemoryApplicationDenial,
> {
    application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
        checkpoint,
        predecessor,
        resources,
        author,
    )
}

#[test]
fn checkpoint_transition_publishes_typed_effects_and_reopens_under_target_roster() {
    let (source, predecessor) = source();
    assert!(
        application_installation::in_memory_program_from_checkpoint(
            validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration(),
            checkpoint::checkpoint_limits(),
            source.clone()
        )
        .is_err(),
        "ordinary restore must refuse the retired activation"
    );
    let migrated = transition(
        source,
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new("migrated-record").unwrap(),
                )
                .field(UnrelatedValueField::reference(), 73_u64),
            )
        },
    )
    .expect("acknowledged native transition must install a World with successor authority");
    assert!(migrated
        .on_branch(migrated.current_world())
        .select()
        .is_ok());
    let checkpoint = migrated.capture_application_checkpoint().unwrap();
    drop(migrated);
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("ordinary restore must recognize the target activation");
    assert!(reopened
        .on_branch(reopened.current_world())
        .select()
        .is_ok());
    // Resolve the actual migrated row through target World authority; capture
    // alone would not prove that typed effects survived native publication.
    let selected = reopened
        .on_branch(reopened.current_world())
        .select()
        .unwrap();
    assert!(selected
        .resolve_entity(
            UnrelatedValueField::reference(),
            73_u64,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
        )
        .is_ok());
}

#[test]
fn checkpoint_transition_predecessor_mismatch_and_selection_bound_deny_before_authoring() {
    let (source, _) = source();
    for (predecessor, resources) in [
        (
            Predecessor::new(&"0".repeat(64)).unwrap(),
            Resources::bounded(512, 32, 4096).unwrap(),
        ),
        (
            source_predecessor(),
            Resources::bounded(1, 32, 4096).unwrap(),
        ),
    ] {
        let mut called = false;
        assert!(transition(source.clone(), predecessor, resources, |_, _| {
            called = true;
            Ok(())
        })
        .is_err());
        assert!(
            !called,
            "source and scope preflight must precede typed authoring"
        );
    }
}

fn source_predecessor() -> Predecessor {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    Predecessor::new(&program.revision().to_string()).unwrap()
}

#[test]
fn checkpoint_transition_cannot_swallow_authoring_resource_refusal() {
    let (source, predecessor) = source();
    let result = transition(
        source,
        predecessor,
        Resources::bounded(512, 1, 4096).unwrap(),
        |writer, _| {
            let _ = writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new("over-budget-record")
                        .unwrap(),
                )
                .field(UnrelatedValueField::reference(), 73_u64),
            );
            Ok(())
        },
    );
    assert!(
        result.is_err(),
        "an ignored refusal cannot publish partial migration effects"
    );
}

#[test]
fn checkpoint_transition_bounds_retained_key_capacity_even_for_a_short_key() {
    let (source, predecessor) = source();
    let result = transition(
        source,
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            let mut key = String::with_capacity(64 * 1024);
            key.push_str("short-key");
            writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap(),
                )
                .field(UnrelatedValueField::reference(), 73_u64),
            )
        },
    );
    let denial = result
        .err()
        .expect("small visible key must not hide retained seed allocation");
    assert!(format!("{denial:?}").contains("authoring resources exceeded"));
}

#[test]
fn checkpoint_transition_deferred_settlement_retains_repair_without_rerunning_authoring() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let denial = transition(
        source,
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            calls += 1;
            writer.bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new("repaired-record").unwrap(),
                )
                .field(UnrelatedValueField::reference(), 89_u64),
            )?;
            writer.fail_next_durable_append_for_test();
            Ok(())
        },
    )
    .err()
    .expect("performed but unacknowledged transition must expose no World");
    let application_installation::WorthQueryInMemoryApplicationDenial::CheckpointTransitionDeferred(
        pending,
    ) = denial
    else {
        panic!("expected exact native repair custody, got {denial:?}");
    };
    pending.fail_next_durable_append_for_test();
    let pending = pending
        .repair_to_checkpoint()
        .expect_err("a refused repair retains the same capsule");
    let checkpoint = pending
        .repair_to_checkpoint()
        .expect("native repair acknowledges the existing performed transition");
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
        checkpoint,
    )
    .expect("only repaired target checkpoint may create a World");
    let selected = reopened
        .on_branch(reopened.current_world())
        .select()
        .unwrap();
    assert!(selected
        .resolve_entity(
            UnrelatedValueField::reference(),
            89_u64,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
        )
        .is_ok());
    assert_eq!(calls, 1, "repair must not rerun typed migration authoring");
}

#[test]
fn checkpoint_transition_retains_acknowledged_checkpoint_after_later_installation_denial() {
    let (source, predecessor) = source();
    let (mut configuration,) = configuration();
    configuration.install_route = false;
    let denial =
        application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
            validated_program(),
            application_installation::WorthQueryApplicationProgramRoster::new(),
            TemporalHostSchema::declaration().unwrap(),
            (configuration,),
            checkpoint::checkpoint_limits(),
            source,
            predecessor,
            Resources::bounded(512, 32, 4096).unwrap(),
            |writer, _| {
                writer.bind_entity(
                    primary_graph::WorthQueryApplicationEntitySeed::new(
                        UnrelatedRecord::reference(),
                        primary_graph::WorthQueryApplicationEntityKey::new("acknowledged-record")
                            .unwrap(),
                    )
                    .field(UnrelatedValueField::reference(), 101_u64),
                )
            },
        )
        .err()
        .expect(
            "incomplete conditional routes must refuse installation after native acknowledgment",
        );
    let application_installation::WorthQueryInMemoryApplicationDenial::CheckpointTransitionAcknowledged { checkpoint: target, cause } = denial else { panic!("acknowledged target effects must retain recovery custody: {denial:?}"); };
    assert!(matches!(
        cause.as_ref(),
        application_installation::WorthQueryInMemoryApplicationDenial::ConditionalPublication(_)
    ));
    let reopened = application_installation::in_memory_program_from_checkpoint(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        self::configuration(),
        checkpoint::checkpoint_limits(),
        target,
    )
    .expect("corrected configuration ordinary-restores actual acknowledged target effects");
    let selected = reopened
        .on_branch(reopened.current_world())
        .select()
        .unwrap();
    assert!(selected
        .resolve_entity(
            UnrelatedValueField::reference(),
            101_u64,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
        )
        .is_ok());
}
