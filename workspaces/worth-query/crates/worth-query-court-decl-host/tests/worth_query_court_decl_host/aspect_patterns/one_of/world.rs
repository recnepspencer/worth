use super::*;
pub struct Program;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "native-record-patterns.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, Feature, rule::OneOf>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("native-record-patterns.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<authoring::Binding>()
            .finish_action()
            .native_action::<retirement::Binding>()
            .finish_action()
            .native_action::<restore_first::Binding>()
            .finish_action()
            .native_action::<retired_fields::retire::Binding>()
            .finish_action()
            .native_action::<retired_fields::restore::Binding>()
            .finish_action()
            .native_action::<sparse::Binding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = Option<WorthQueryCancellationSource>;
    fn configure(
        cancellation: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<authoring::Binding, _>(authoring::Handler {
            cancel_after_candidate: cancellation,
        })?;
        setup.handler::<retirement::Binding, _>(retirement::Handler)?;
        setup.handler::<restore_first::Binding, _>(restore_first::Handler)?;
        setup.handler::<sparse::Binding, _>(sparse::Handler)?;
        setup.handler::<retired_fields::retire::Binding, _>(retired_fields::retire::Handler)?;
        setup.handler::<retired_fields::restore::Binding, _>(retired_fields::restore::Handler)?;
        setup.invariant(
            rule::OneOf::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            rule::resolve,
        )
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
fn program() -> ValidatedApplicationProgram<Schema, Program> {
    ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap()
}
fn build() -> ApplicationBuildIdentity {
    ApplicationBuildIdentity::parse("919b0000-0000-0000-0000-000000000001").unwrap()
}
pub fn install() -> Host {
    install_with_control(None)
}
pub fn install_with_control(cancellation: Option<WorthQueryCancellationSource>) -> Host {
    try_install_seed(cancellation, Some(11)).unwrap()
}
pub fn try_install_seed(
    cancellation: Option<WorthQueryCancellationSource>,
    first: Option<u64>,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    try_install_seed_with_neighbour(cancellation, first, false, false)
}
pub fn install_with_neighbour() -> Host {
    try_install_seed_with_neighbour(None, Some(11), true, false).unwrap()
}
pub fn install_retirement() -> Host {
    try_install_seed_with_neighbour(None, Some(11), true, true).unwrap()
}
fn try_install_seed_with_neighbour(
    cancellation: Option<WorthQueryCancellationSource>,
    first: Option<u64>,
    neighbour: bool,
    retirement: bool,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    in_memory_program(
        program(),
        Schema::declaration().unwrap(),
        (cancellation,),
        build(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            let mut seed = WorthQueryApplicationEntitySeed::new(
                    Record::reference(),
                    WorthQueryApplicationEntityKey::new("record").unwrap(),
                )
                .field(Key::at("Key"), "record".to_owned());
            if let Some(first) = first { seed = seed.field(FirstValue::at("FirstValue"), first); }
            if retirement { seed = seed.field(sparse::annotation::AnnotationValue::at("Value"), 21); }
            graph.bind_entity(seed)?;
            if neighbour {
                let seed = WorthQueryApplicationEntitySeed::new(
                    Record::reference(), WorthQueryApplicationEntityKey::new("neighbour").unwrap())
                    .field(Key::at("Key"), "neighbour".to_owned());
                let seed = if retirement {
                    seed.field(ThirdValue::at("ThirdValue"), 33)
                        .field(sparse::annotation::AnnotationValue::at("Value"), 81)
                        .field(sparse::annotation::AnnotationNote::at("Note"), 77)
                } else { seed.field(FirstValue::at("FirstValue"), 99) };
                graph.bind_entity(seed)?;
            }
            Ok(())
        },
    )
}
pub fn restore(checkpoint: WorthQueryApplicationCheckpoint) -> Host {
    try_restore(checkpoint).unwrap()
}
pub fn try_restore(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    WorthQueryInterimCheckpointRestore::program(
        program(),
        Schema::declaration().unwrap(),
        (None,),
        build(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        checkpoint,
    )
}
pub fn read(host: &Host) -> query::Row {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .query(query::Read {
            key: "record".into(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
pub fn try_edit(
    host: &Host,
    change: authoring::Change,
    key: &str,
) -> Result<
    WorthQueryApplicationMutationOutcome<authoring::Denial, authoring::Edited>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(authoring::Edit {
            key: "record".into(),
            change,
        })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}

pub fn edit(
    host: &Host,
    change: authoring::Change,
    key: &str,
) -> WorthQueryApplicationMutationOutcome<authoring::Denial, authoring::Edited> {
    try_edit(host, change, key).unwrap()
}
pub fn restore_variant(
    host: &Host,
    row: query::Row,
    key: &str,
) -> WorthQueryApplicationMutationOutcome<authoring::Denial, authoring::Edited> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(restore_first::Restore {
            key: "record".into(),
            row,
        })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
pub fn pin(
    host: &Host,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .retain_current_read()
        .unwrap()
}

pub fn checkpoint(host: &Host) -> WorthQueryApplicationCheckpoint {
    host.runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap()
}
