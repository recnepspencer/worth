//! Real native predecessor -> adopted home -> ordinary target reopen journey.
use super::*;
use application_installation::{
    ApplicationHome, WorthQueryApplicationOpenRefusal as Refusal, WorthQueryHomeOpening as Opening,
    WorthQueryOpenAdoption as Adoption, WorthQueryOpenAdoptionPredecessor as Predecessor,
    WorthQueryOpenAdoptionResources as Resources, WorthQueryRefusedHome as RefusedHome,
};

pub(super) type Target = application_installation::WorthQueryProgramApplicationRuntime<
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

pub(super) fn source() -> (
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

pub(super) fn reopen(home: ApplicationHome) -> Result<Target, Refusal> {
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

pub(super) fn record(
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
pub(super) fn assert_record_resolves(application: &Target, value: u64) {
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
fn an_unmatched_rostered_image_resumes_without_adoption_preflight() {
    let (source, _) = source();
    let legacy = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    let recorded = *legacy.revision();
    let mut called = false;
    let resumed = application_installation::program(
        validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        checkpoint::checkpoint_limits(),
    )
    .roster(application_installation::WorthQueryApplicationProgramRoster::new().support(legacy))
    .adopt_on_open(Adoption::new(
        Predecessor::new(&"0".repeat(64)).unwrap(),
        Resources::bounded(1, 1, 1).unwrap(),
        |_, _| {
            called = true;
            Ok(())
        },
    ))
    .open(home_of(&source))
    .expect("an unmatched rostered image resumes without adoption preflight");
    assert_eq!(
        resumed.opening(),
        &Opening::Resumed {
            installed: recorded
        }
    );
    assert!(!called);
}

#[test]
fn open_adoption_selection_bound_denies_before_authoring() {
    let (source, predecessor) = source();
    let mut called = false;
    let refusal = adopt(
        home_of(&source),
        configuration(),
        predecessor,
        Resources::bounded(1, 32, 4096).unwrap(),
        |_, _| {
            called = true;
            Ok(())
        },
    )
    .err()
    .expect("source scope preflight must refuse the open");
    assert!(matches!(refusal.home, RefusedHome::Unchanged(_)));
    assert!(!called, "scope preflight must precede typed authoring");
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

pub(super) fn adopt_through_adapter(
    source: application_installation::WorthQueryApplicationCheckpoint,
    configuration: (TemporalContributionConfiguration,),
    predecessor: Predecessor,
    resources: Resources,
    author: impl FnOnce(
        &mut application_installation::WorthQueryOpenAdoptionWriter<'_, TemporalHostSchema>,
        &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<Target, Refusal> {
    application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        configuration,
        checkpoint::checkpoint_limits(),
        source,
        predecessor,
        resources,
        author,
    )
}
