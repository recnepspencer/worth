//! Real native predecessor -> adopted home -> ordinary target reopen journey.
use super::*;
use application_installation::{
    ApplicationHome, WorthQueryApplicationOpenDenial as Denial,
    WorthQueryApplicationOpenRefusal as Refusal, WorthQueryHomeOpening as Opening,
    WorthQueryOpenAdoption as Adoption, WorthQueryOpenAdoptionPredecessor as Predecessor,
    WorthQueryOpenAdoptionResources as Resources, WorthQueryRefusedHome as RefusedHome,
};

type Target = application_installation::WorthQueryProgramApplicationRuntime<
    TemporalHostSchema,
    TemporalInstallationProgram,
>;

pub(super) struct LegacyTemporalProgram;
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

pub(super) fn configuration() -> (TemporalContributionConfiguration,) {
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

/// The home a closed application leaves. Until `close` lands, the image is
/// carried across through the durability byte seam.
pub(super) fn home_of(
    checkpoint: &application_installation::WorthQueryApplicationCheckpoint,
) -> ApplicationHome {
    ApplicationHome::memory_from_image_bytes_for_durability_test(checkpoint.bytes().to_vec())
}

fn source() -> (
    application_installation::WorthQueryApplicationCheckpoint,
    Predecessor,
) {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    let predecessor = Predecessor::new(&program.revision().to_string()).unwrap();
    let application = application_installation::program(
        program,
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .initial_state(|graph, installed| {
        let principal = installed
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        seed_graph(graph, &principal, "checkpoint-transition", 0, 1, true);
        Ok(())
    })
    .open(ApplicationHome::memory())
    .unwrap();
    assert_eq!(application.opening(), &Opening::Started);
    (
        application.capture_application_checkpoint().unwrap(),
        predecessor,
    )
}

fn reopen(home: ApplicationHome) -> Result<Target, Refusal> {
    application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .open(home)
}

fn adopt(
    home: ApplicationHome,
    configuration: (TemporalContributionConfiguration,),
    predecessor: Predecessor,
    resources: Resources,
    author: impl FnOnce(
        &mut application_installation::WorthQueryOpenAdoptionWriter<'_, TemporalHostSchema>,
        &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<Target, Refusal> {
    application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration,
        checkpoint::checkpoint_limits(),
    )
    .adopt_on_open(Adoption::new(predecessor, resources, author))
    .open(home)
}

fn record(
    key: impl Into<String>,
    value: u64,
) -> primary_graph::WorthQueryApplicationEntitySeed<TemporalHostSchema, UnrelatedRecord> {
    primary_graph::WorthQueryApplicationEntitySeed::new(
        UnrelatedRecord::reference(),
        primary_graph::WorthQueryApplicationEntityKey::new(key).unwrap(),
    )
    .field(UnrelatedValueField::reference(), value)
}

/// Resolves the adopted row through target World authority; a reopen alone
/// would not prove that typed effects survived native publication.
fn assert_record_resolves(application: &Target, value: u64) {
    let selected = application
        .on_branch(application.current_world())
        .select()
        .unwrap();
    assert!(selected
        .resolve_entity(
            UnrelatedValueField::reference(),
            value,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary
        )
        .is_ok());
}

#[test]
fn open_adoption_publishes_typed_effects_and_resumes_under_target_roster() {
    let (source, predecessor) = source();
    let refusal = reopen(home_of(&source))
        .err()
        .expect("an open without the adoption must refuse the retired activation");
    assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
    let target = *validated_program().revision();
    let adopted = adopt(
        home_of(&source),
        configuration(),
        predecessor.clone(),
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| writer.bind_entity(record("migrated-record", 73)),
    )
    .expect("acknowledged native adoption must install a World with successor authority");
    assert_eq!(
        adopted.opening(),
        &Opening::Adopted {
            from: predecessor.clone(),
            installed: target,
        }
    );
    assert!(adopted.on_branch(adopted.current_world()).select().is_ok());
    let successor = adopted.capture_application_checkpoint().unwrap();
    drop(adopted);
    let mut called = false;
    let resumed = adopt(
        home_of(&successor),
        configuration(),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |_, _| {
            called = true;
            Ok(())
        },
    )
    .expect("a home already at the target resumes under the same open call");
    assert_eq!(resumed.opening(), &Opening::Resumed { installed: target });
    drop(resumed);
    assert!(!called, "a resumed home must skip the declared adoption");
    assert_record_resolves(&reopen(home_of(&successor)).unwrap(), 73);
}

#[test]
fn open_adoption_predecessor_mismatch_and_selection_bound_deny_before_authoring() {
    let (source, predecessor) = source();
    for (predecessor, resources) in [
        (
            Predecessor::new(&"0".repeat(64)).unwrap(),
            Resources::bounded(512, 32, 4096).unwrap(),
        ),
        (predecessor, Resources::bounded(1, 32, 4096).unwrap()),
    ] {
        let mut called = false;
        let refusal = adopt(
            home_of(&source),
            configuration(),
            predecessor,
            resources,
            |_, _| {
                called = true;
                Ok(())
            },
        )
        .err()
        .expect("source and scope preflight must refuse the open");
        assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
        assert!(
            !called,
            "source and scope preflight must precede typed authoring"
        );
    }
}

#[test]
fn open_adoption_cannot_swallow_authoring_resource_refusal() {
    let (source, predecessor) = source();
    let refusal = adopt(
        home_of(&source),
        configuration(),
        predecessor,
        Resources::bounded(512, 1, 4096).unwrap(),
        |writer, _| {
            let _ = writer.bind_entity(record("over-budget-record", 73));
            Ok(())
        },
    )
    .err()
    .expect("an ignored refusal cannot publish partial adoption effects");
    assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
}

#[test]
fn open_adoption_bounds_retained_key_capacity_even_for_a_short_key() {
    let (source, predecessor) = source();
    let refusal = adopt(
        home_of(&source),
        configuration(),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            let mut key = String::with_capacity(64 * 1024);
            key.push_str("short-key");
            writer.bind_entity(record(key, 73))
        },
    )
    .err()
    .expect("small visible key must not hide retained seed allocation");
    assert!(format!("{:?}", refusal.denial).contains("authoring resources exceeded"));
}

#[test]
fn open_adoption_deferred_settlement_holds_the_home_in_repair_without_rerunning_authoring() {
    let (source, predecessor) = source();
    let mut calls = 0;
    let refusal = adopt(
        home_of(&source),
        configuration(),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| {
            calls += 1;
            writer.bind_entity(record("repaired-record", 89))?;
            writer.fail_next_durable_append_for_test();
            Ok(())
        },
    )
    .err()
    .expect("performed but unacknowledged adoption must expose no World");
    assert!(matches!(refusal.denial, Denial::AdoptionDeferred(_)));
    let pending = match refusal.home {
        RefusedHome::InRepair(pending) => pending,
        other => panic!("expected exact native repair custody, got {other:?}"),
    };
    pending.fail_next_durable_append_for_test();
    let pending = pending
        .repair()
        .expect_err("a refused repair retains the same capsule");
    let home = pending
        .repair()
        .expect("native repair acknowledges the existing performed adoption");
    let reopened = reopen(home).expect("only the repaired home may create a World");
    assert_eq!(
        reopened.opening(),
        &Opening::Resumed {
            installed: *validated_program().revision()
        }
    );
    assert_record_resolves(&reopened, 89);
    assert_eq!(calls, 1, "repair must not rerun typed adoption authoring");
}

#[test]
fn open_adoption_returns_the_successor_home_after_a_later_installation_denial() {
    let (source, predecessor) = source();
    let (mut incomplete,) = configuration();
    incomplete.install_route = false;
    let refusal = adopt(
        home_of(&source),
        (incomplete,),
        predecessor,
        Resources::bounded(512, 32, 4096).unwrap(),
        |writer, _| writer.bind_entity(record("acknowledged-record", 101)),
    )
    .err()
    .expect("incomplete conditional routes must refuse the open after native acknowledgment");
    assert!(matches!(refusal.denial, Denial::ConditionalPublication(_)));
    let home = match refusal.home {
        RefusedHome::Successor(home) => home,
        other => panic!("acknowledged adoption effects must return the successor home: {other:?}"),
    };
    let reopened =
        reopen(home).expect("a corrected configuration resumes the actual acknowledged successor");
    assert_record_resolves(&reopened, 101);
}
