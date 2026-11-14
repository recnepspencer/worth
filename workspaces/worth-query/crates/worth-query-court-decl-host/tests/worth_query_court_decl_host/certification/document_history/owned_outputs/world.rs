//! Separate installed world: model `a` (value 1) owning member `m`, plus NAV grant.
use super::*;
pub(super) mod authorization_clock;
use worth_query_host::facade::product::WorthQueryProductBranch;
pub type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
pub type Document<'a> = WorthQueryApplicationHistoryDocument<'a, Schema, History>;
#[derive(Clone, Copy, Default)]
pub struct Configuration {
    pub second_derivation: Option<fn(u64) -> u64>,
    pub child_derivation: Option<fn(u64) -> u64>,
    pub reconciliation_indices: bool,
    pub reconciliation_peer_caps: bool,
    pub reconciliation_emission: bool,
    pub envelope_records: bool,
    pub interleaved_records: bool,
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        synchronous_edit::reconciliation::contracts(contracts)?;
        family::decision_write::contracts(contracts)?;
        family::convergent::contracts(contracts)?;
        family::decision_input::contracts(contracts)?;
        family::parity_input::contracts(contracts)?;
        family::ordinary_input::contracts(contracts)?;
        family::diamond::contracts(contracts)?;
        family::predecessor_family::contracts(contracts)?;
        from_first::contracts(contracts)?;
        restricted_from_first::contracts(contracts)?;
        first::contracts(contracts)?;
        second::contracts(contracts)?;
        child::contracts(contracts)?;
        restricted::contracts(contracts)?;
        upstream::contracts(contracts)?;
        downstream::contracts(contracts)?;
        restricted_owned::contracts(contracts)
    }
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        synchronous_edit::reconciliation::install(
            setup,
            configuration.reconciliation_indices,
            configuration.reconciliation_peer_caps,
            configuration.reconciliation_emission,
        )?;
        setup.handler::<authored_member::retire::Binding, _>(authored_member::retire::Handler)?;
        setup.handler::<authored_member::restore::Binding, _>(authored_member::restore::Handler)?;
        setup.handler::<edit::Binding, _>(edit::Handler)?;
        setup.handler::<synchronous_edit::new_source::restricted::Binding, _>(
            synchronous_edit::new_source::restricted::Handler,
        )?;
        synchronous_edit::envelope_fixture::install(setup)?;
        setup.handler::<synchronous_edit::Binding, _>(synchronous_edit::Handler)?;
        setup.handler::<synchronous_edit::new_source::Binding, _>(
            synchronous_edit::new_source::Handler,
        )?;
        setup.handler::<predecessor::synchronous::Binding, _>(predecessor::synchronous::Handler)?;
        setup.handler::<member_edit::Binding, _>(member_edit::Handler)?;
        setup.handler::<member_delete::Binding, _>(member_delete::Handler)?;
        family::decision_write::install(setup)?;
        family::convergent::install(setup)?;
        family::decision_input::install(setup)?;
        family::parity_input::install(setup)?;
        family::ordinary_input::install(setup)?;
        family::diamond::install(setup)?;
        family::predecessor_family::install(setup)?;
        from_first::install(setup)?;
        restricted_from_first::install(setup)?;
        first::install(setup)?;
        match configuration.second_derivation {
            Some(derive) => second::install_with_derivation(setup, derive)?,
            None => second::install(setup)?,
        }
        match configuration.child_derivation {
            Some(derive) => child::install_with_derivation(setup, derive)?,
            None => child::install(setup)?,
        }
        restricted::install(setup)?;
        upstream::install(setup)?;
        downstream::install(setup)?;
        restricted_owned::install(setup)
    }
}
pub fn install() -> Host {
    install_with(host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    ))
}
/// The same world whose every request is a worker-only lease without accounting.
pub fn install_worker_only() -> Host {
    install_with(worker_limits())
}
pub(super) fn worker_limits() -> WorthQueryApplicationLimits {
    worker_limits_with_clock(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}
pub(super) fn worker_limits_with_clock(
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> WorthQueryApplicationLimits {
    use worth_query_host::facade::runtime::*;
    let world =
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
            clock,
        )
        .with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(8).unwrap(),
        ));
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(128).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}
fn install_with(limits: WorthQueryApplicationLimits) -> Host {
    install_program::<program::Program>(limits)
}
pub(super) fn install_program<P>(
    limits: WorthQueryApplicationLimits,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    install_program_with_value::<P>(limits, 1)
}
pub(super) fn install_program_with_value<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    install_program_with_other_model::<P>(limits, initial_value, None)
}
/// Seed an independent model for cross-document demand-correspondence courts.
pub(super) fn install_program_with_other_model<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
    other_value: Option<u64>,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    install_configured::<P>(limits, initial_value, other_value, Configuration::default())
}
pub(super) fn install_configured<P>(
    limits: WorthQueryApplicationLimits,
    initial_value: u64,
    other_value: Option<u64>,
    configuration: Configuration,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, P>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (configuration,),
        ApplicationBuildIdentity::parse(
            match (
                configuration.reconciliation_indices,
                configuration.reconciliation_peer_caps,
                configuration.reconciliation_emission,
            ) {
                (false, false, false) => "91830000-0000-0000-0000-000000005356",
                (true, false, false) => "91830000-0000-0000-0000-000000005358",
                (false, true, false) => "91830000-0000-0000-0000-000000005359",
                (false, false, true) => "91830000-0000-0000-0000-00000000535a",
                (true, true, false) => "91830000-0000-0000-0000-00000000535b",
                (true, false, true) => "91830000-0000-0000-0000-00000000535c",
                (false, true, true) => "91830000-0000-0000-0000-00000000535d",
                (true, true, true) => "91830000-0000-0000-0000-00000000535e",
            },
        )
        .unwrap(),
        limits,
        |_, graph, installed| {
            seed(graph, installed, initial_value, other_value)?;
            if configuration.envelope_records {
                synchronous_edit::envelope_fixture::seed(graph)?;
            }
            if configuration.interleaved_records {
                synchronous_edit::interleaved::seed(graph)?;
            }
            if configuration.reconciliation_peer_caps {
                synchronous_edit::reconciliation::peer_caps::seed(graph)?;
            }
            Ok(())
        },
    )
    .unwrap()
}
/// Attach the model history on a fresh owned head forked from the current world.
pub fn attach<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
) -> Document<'_> {
    let scope = request_scope();
    attach_with_scope(host, &scope)
}
pub(super) fn attach_with_scope<'a, P: ApplicationProgramDefinition<Schema>>(
    host: &'a WorthQueryProgramApplicationRuntime<Schema, P>,
    scope: &WorthQueryRequestScope,
) -> Document<'a> {
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
        let head = host
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head();
        host.request(&principal, scope)
            .on_branch(head.branch())
            .attach_history::<History>(&phase, head, query::model::Read("a".into()))
            .unwrap()
    })
    .unwrap()
}
/// Independent public reads of the actual rows on one branch.
pub fn model(host: &Host, branch: WorthQueryProductBranch) -> query::Row {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
pub fn outputs(host: &Host, branch: WorthQueryProductBranch) -> (query::Row, query::Row) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope).on_branch(branch);
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let second = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    (first, second)
}

pub fn restore_changed_build(checkpoint: WorthQueryApplicationCheckpoint, worker: bool) -> Host {
    restore_program_checkpoint::<program::Program>(checkpoint, worker, true)
}
pub fn restore_same_build(checkpoint: WorthQueryApplicationCheckpoint, worker: bool) -> Host {
    restore_program_checkpoint::<program::Program>(checkpoint, worker, false)
}
pub(super) fn restore_program_checkpoint<P>(
    checkpoint: WorthQueryApplicationCheckpoint,
    worker: bool,
    changed: bool,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, P>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let limits = if worker {
        worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    WorthQueryInterimCheckpointRestore::program(
        program,
        schema,
        (Configuration {
            second_derivation: if changed {
                Some(|value| value + 2000)
            } else {
                None
            },
            ..Default::default()
        },),
        ApplicationBuildIdentity::parse(if changed {
            "91830000-0000-0000-0000-000000005357"
        } else {
            "91830000-0000-0000-0000-000000005356"
        })
        .unwrap(),
        limits,
        checkpoint,
    )
    .unwrap()
}

pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    initial_value: u64,
    other_value: Option<u64>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed.principal_binding(Actor::reference()).unwrap(),
        WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
        1,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("a").unwrap(),
        )
        .field(ModelKey::at("ModelKey"), "a".to_owned())
        .field(ModelValue::at("ModelValue"), initial_value),
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new("m").unwrap(),
        )
        .field(MemberKey::at("MemberKey"), "m".to_owned())
        .field(MemberValue::at("MemberValue"), 5),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        ModelMember::reference(),
        "owns:a",
        WorthQueryApplicationEntityKey::new("a").unwrap(),
        WorthQueryApplicationEntityKey::new("m").unwrap(),
    ))?;
    if let Some(value) = other_value {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new("b").unwrap(),
            )
            .field(ModelKey::at("ModelKey"), "b".to_owned())
            .field(ModelValue::at("ModelValue"), value),
        )?;
    }
    navigation::seed(graph)
}
