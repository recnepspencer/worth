pub struct Program;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "class-read-cost.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("class-read-cost.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<Binding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<Binding, _>(Handler)
    }
}
fn install() -> WorthQueryProgramApplicationRuntime<Schema, Program> {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("919b0000-0000-0000-0000-000000000002").unwrap(),
        host_limits(),
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
                .field(Key::reference(), "record".to_owned())
                .field(Sink::reference(), 0)
                .field(Value3::reference(), 33),
            )?;
            Ok(())
        },
    )
    .unwrap()
}
pub fn measure() -> [WorthQueryInvariantProjectionWork; 3] {
    [Mode::Present, Mode::Absent, Mode::Discover].map(|mode| {
        let host = install();
        let meanings = host.installed_schema().value_meaning().unwrap();
        let classes: Vec<_> = meanings
            .iter()
            .filter(|aspect| aspect.entity() == "Record" && aspect.aspect().starts_with("Class"))
            .map(|aspect| aspect.aspect())
            .collect();
        assert_eq!(
            classes.len(),
            CLASSES,
            "actual installed/native checked roster"
        );
        for (n, name) in (1..=CLASSES).zip(&classes) {
            assert!(classes.contains(&format!("Class{n}").as_str()), "{name}");
        }
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        assert_eq!(
            host.runtime()
                .request(&principal, &scope)
                .query(readback::Read)
                .execute()
                .unwrap()
                .rows()[0]
                .sink,
            0
        );
        let nonce = match mode {
            Mode::Present => 1,
            Mode::Absent => 2,
            Mode::Discover => 3,
        };
        let report = host
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(mode.clone())
            .without_source()
            .idempotency(&nonce)
            .execute_in_program_report(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        let WorthQueryMutationHandlerWork::Captured(work) = report.decision_work() else {
            panic!("must capture this invocation");
        };
        let work = work.projection_work();
        let outcome = report.into_outcome().unwrap();
        match mode {
            Mode::Absent => assert!(
                matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::DomainDenied(Denial::Absent)
                ),
                "{outcome:?}"
            ),
            Mode::Present | Mode::Discover => {
                let WorthQueryApplicationMutationOutcome::Committed { result, .. } = outcome else {
                    panic!("{outcome:?}");
                };
                assert_eq!(result.0, 33);
                assert_eq!(
                    host.runtime()
                        .request(&principal, &scope)
                        .query(readback::Read)
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .sink,
                    33
                );
            }
        }
        work
    })
}
