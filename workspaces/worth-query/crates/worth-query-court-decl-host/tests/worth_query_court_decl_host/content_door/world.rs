use super::*;
pub struct Program;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "content-door.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, Feature, source::CheckSamples>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("content-door.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<source::expectation::Binding>()
            .finish_action()
            .native_action::<created_content::Binding>()
            .finish_action()
            .native_action::<revision::Binding>()
            .finish_action()
            .native_action::<operation::Binding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = (
        Option<WorthQueryCancellationSource>,
        std::sync::Arc<source::Calls>,
    );
    fn configure(
        (cancellation, calls): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup
            .managed_index_provider::<source::Index, _>(source::Provider(calls.clone()))
            .with_refinement()
            .install()?;
        setup.invariant(
            source::CheckSamples::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| source::resolve(resolver, calls.clone()),
        )?;
        setup.handler::<source::expectation::Binding, _>(source::expectation::Handler)?;
        setup.handler::<created_content::Binding, _>(created_content::Handler)?;
        setup.handler::<revision::Binding, _>(revision::Handler)?;
        setup.handler::<operation::Binding, _>(operation::Handler { cancellation })
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn install() -> Host {
    install_with_control(None)
}
pub fn install_with_control(cancellation: Option<WorthQueryCancellationSource>) -> Host {
    install_with_calls(cancellation, Default::default())
}
pub fn install_with_calls(
    cancellation: Option<WorthQueryCancellationSource>,
    calls: std::sync::Arc<source::Calls>,
) -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((cancellation, calls),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Record::reference(),
                    WorthQueryApplicationEntityKey::new("record").unwrap(),
                )
                .field(Key::reference(), "record".to_owned()),
            )?;
            Ok(())
        },
    )
    .unwrap()
}
pub fn try_edit(
    host: &Host,
    mode: u64,
    samples: Vec<i64>,
    reference: Vec<u8>,
    key: &str,
) -> Result<
    WorthQueryApplicationMutationOutcome<operation::Denial, operation::Edited>,
    WorthQueryApplicationRequestMutationDenial,
> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(operation::Edit {
            mode,
            samples,
            reference,
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
    mode: u64,
    samples: Vec<i64>,
    reference: Vec<u8>,
    key: &str,
) -> WorthQueryApplicationMutationOutcome<operation::Denial, operation::Edited> {
    try_edit(host, mode, samples, reference, key).unwrap()
}
pub fn observe(host: &Host, key: &str) -> Option<(ContentRefId, Vec<u8>)> {
    let outcome = edit(host, 99, vec![], vec![], key);
    let WorthQueryApplicationMutationOutcome::Committed { result, .. } = outcome else {
        panic!("content observation must complete: {outcome:?}");
    };
    result.before
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
pub fn restore(
    checkpoint: WorthQueryApplicationCheckpoint,
    calls: std::sync::Arc<source::Calls>,
) -> Host {
    WorthQueryInterimCheckpointRestore::program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((None, calls),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(WorthQueryAdmissionClock::system()),
        checkpoint,
    )
    .unwrap()
}
