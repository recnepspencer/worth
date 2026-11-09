use super::*;
pub type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
pub type Retained =
    WorthQuerySettledProgramOutputOperation<Schema, program::Program, program::Graph>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fault {
    None,
    OmitRightRole,
    CorruptInputMember,
    RetainAuthoredAnchor,
    OmitRestoredAnchor,
    ChangedRestoredKey,
    RepeatedRestoredMember,
    ForeignRestoredMember,
    ExtraRestoredMember,
    ChangedRestoredValue,
}
pub struct Configuration {
    pub fault: Fault,
    pub hashes: Hashes,
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        producer::contracts(contracts)
    }
    fn configure(
        config: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        roster::configure(setup, config.fault)?;
        setup.handler::<edit::source::Binding, _>(edit::source::Handler)?;
        setup.handler::<edit::generated::Binding, _>(edit::generated::Handler)?;
        setup.handler::<generation::Binding, _>(generation::Handler {
            omit_right_role: config.fault == Fault::OmitRightRole,
        })?;
        setup.producer::<producer::Producer>(producer::Provider(config.hashes, config.fault))?;
        setup.conditional::<readiness::Readiness>(())
    }
}
pub fn install(fault: Fault, hashes: Hashes) -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, program::Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        (Configuration { fault, hashes },),
        ApplicationBuildIdentity::parse("919b0000-0000-0000-0000-000000000003").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for (key, value) in [("source", 1), ("left", 11), ("right", 22)] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Source::reference(),
                        WorthQueryApplicationEntityKey::new(key).unwrap(),
                    )
                    .field(SourceKey::at("SourceKey"), key.to_owned())
                    .field(SourceValue::at("SourceValue"), value),
                )?;
            }
            for key in ["left", "right"] {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    SourceMember::reference(),
                    format!("member:{key}"),
                    WorthQueryApplicationEntityKey::new("source").unwrap(),
                    WorthQueryApplicationEntityKey::new(key).unwrap(),
                ))?;
            }
            Ok(())
        },
    )
    .unwrap()
}
pub fn source(request: &Request<'_, '_, '_>) -> GraphRow {
    request
        .query(graph_read::source::Read("source".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
pub fn generated(request: &Request<'_, '_, '_>) -> GraphRow {
    request
        .query(graph_read::generated::Read("generated".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
pub fn edit_source(
    host: &Host,
    request: &Request<'_, '_, '_>,
    value: u64,
) -> WorthQueryApplicationCommitReceipt {
    let outcome = request
        .advanced()
        .mutate(edit::source::Intent(edit::Edit {
            key: "left".into(),
            value,
        }))
        .without_source()
        .idempotency(&format!("source:{value}"))
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}");
    };
    receipt
}
pub fn corrupt_child(host: &Host, request: &Request<'_, '_, '_>) {
    let outcome = request
        .advanced()
        .mutate(edit::generated::Intent(edit::Edit {
            key: "right".into(),
            value: 999,
        }))
        .without_source()
        .idempotency(&"corrupt-child".to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
pub fn settle(host: &Host, request: &Request<'_, '_, '_>) -> Retained {
    let mut operation = request
        .demand(producer::Demand)
        .retain_program_outputs::<program::Program, program::Graph>(host)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()));
    for _ in 0..32 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => return settled,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("refused: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("terminal: {:?}", terminal.denial())
            }
        }
    }
    panic!("small fixed graph did not settle within32 handoffs");
}
