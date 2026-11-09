//! Install, edit, read and close through the public application owners.
mod limits;
use super::{authentication::*, edit, other_edit, program::*, query, schema::*};
pub(super) use limits::{installation_limits, worker_limits};
use worth_query_host::facade::{
    application_contribution::*,
    application_entry::*,
    application_installation::*,
    declaration::{application_program::*, authentication::*},
    primary_graph::*,
    runtime,
};

pub use worth_query_testkit_document_retention::facade::document_world_resources as world_resources;
pub type Host = WorthQueryProgramApplicationRuntime<NumericSchema, Program>;
pub type OutputHost =
    WorthQueryProgramApplicationRuntime<NumericSchema, super::output_program::OutputProgram>;
impl WorthQueryApplicationContribution<NumericSchema> for NumericContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts
            .producer::<super::causal_inspection::output_equivalence::calculation::Producer>()?;
        contracts
            .conditional::<super::causal_inspection::output_equivalence::readiness::Readiness>()?;
        contracts.producer::<super::causal_inspection::retired_outputs::calculation::Producer>()?;
        contracts
            .conditional::<super::causal_inspection::retired_outputs::readiness::Readiness>()?;
        contracts.producer::<super::calculation::Producer>()?;
        contracts.producer::<super::deferred_completion::reciprocal::Producer>()?;
        contracts.conditional::<super::deferred_completion::readiness::Readiness>()?;
        contracts.producer::<super::aspect_input::calculation::Producer>()?;
        contracts.producer::<super::scalar_input::calculation::Producer>()?;
        contracts.conditional::<super::scalar_input::readiness::Readiness>()?;
        contracts.conditional::<super::aspect_input::readiness::Readiness>()?;
        contracts.conditional::<super::calculation_readiness::Readiness>()?;
        contracts.conditional::<super::composite_definition_runtime::Conditional>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<super::write_admission::Binding, _>(super::write_admission::Handler)?;
        setup.handler::<super::causal_inspection::distinct_subjects::grant_status::Binding, _>(
            super::causal_inspection::distinct_subjects::grant_status::Handler,
        )?;
        setup.handler::<super::causal_inspection::output_equivalence::calculation::Binding, _>(
            super::causal_inspection::output_equivalence::calculation::Handler,
        )?;
        setup.producer::<super::causal_inspection::output_equivalence::calculation::Producer>(
            super::causal_inspection::output_equivalence::calculation::Provider,
        )?;
        setup
            .conditional::<super::causal_inspection::output_equivalence::readiness::Readiness>(())?;
        setup.handler::<super::causal_inspection::retired_outputs::calculation::Binding, _>(
            super::causal_inspection::retired_outputs::calculation::Handler,
        )?;
        setup.producer::<super::causal_inspection::retired_outputs::calculation::Producer>(
            super::causal_inspection::retired_outputs::calculation::Provider,
        )?;
        setup.conditional::<super::causal_inspection::retired_outputs::readiness::Readiness>(())?;
        setup.handler::<super::deferred_completion::reciprocal::Binding, _>(
            super::deferred_completion::reciprocal::Handler,
        )?;
        setup.producer::<super::deferred_completion::reciprocal::Producer>(
            super::deferred_completion::reciprocal::Provider,
        )?;
        setup.conditional::<super::deferred_completion::readiness::Readiness>(())?;
        setup.handler::<super::calculation::Binding, _>(super::calculation::Handler)?;
        setup.handler::<super::scalar_input::authoring::Binding, _>(
            super::scalar_input::authoring::Handler,
        )?;
        setup.handler::<super::scalar_input::calculation::Binding, _>(
            super::scalar_input::calculation::Handler,
        )?;
        setup.producer::<super::scalar_input::calculation::Producer>(
            super::scalar_input::calculation::Provider,
        )?;
        setup.conditional::<super::scalar_input::readiness::Readiness>(())?;
        setup.handler::<super::aspect_input::authoring::Binding, _>(
            super::aspect_input::authoring::Handler,
        )?;
        setup.handler::<super::aspect_input::calculation::Binding, _>(
            super::aspect_input::calculation::Handler,
        )?;
        setup.producer::<super::aspect_input::calculation::Producer>(
            super::aspect_input::calculation::Provider,
        )?;
        setup.conditional::<super::aspect_input::readiness::Readiness>(())?;
        setup.producer::<super::calculation::Producer>(super::calculation::Provider)?;
        setup.conditional::<super::calculation_readiness::Readiness>(())?;
        setup.conditional::<super::composite_definition_runtime::Conditional>(())?;
        setup.handler::<super::composite_definition_model::Binding, _>(
            super::composite_definition_handler::Handler,
        )?;
        setup.handler::<edit::EditBinding, _>(edit::Handler)?;
        setup.handler::<super::operand_edit::OperandBinding, _>(super::operand_edit::Handler)?;
        setup.handler::<super::observed_clone::Binding, _>(
            super::authored_identity_handlers::Handler,
        )?;
        setup.handler::<super::adjacency_link::Binding, _>(super::adjacency_link::Handler)?;
        setup.handler::<super::authored_identity_model::CreateBinding, _>(
            super::authored_identity_handlers::Handler,
        )?;
        setup.handler::<super::authored_identity_model::RetireBinding, _>(
            super::authored_identity_handlers::Handler,
        )?;
        setup.handler::<super::authored_identity_model::RestoreBinding, _>(
            super::authored_identity_handlers::Handler,
        )?;
        setup.handler::<super::authored_identity_reference::Binding, _>(
            super::authored_identity_reference::Handler,
        )?;
        setup.handler::<super::source_output_model::IssueBinding, _>(
            super::source_output_handlers::IssueHandler,
        )?;
        setup.handler::<super::source_output_model::RecordBinding, _>(
            super::source_output_handlers::RecordHandler,
        )?;
        setup.handler::<super::source_output_model::RetireBinding, _>(
            super::source_output_handlers::RetireHandler,
        )?;
        setup.handler::<other_edit::OtherBinding, _>(other_edit::Handler)?;
        setup.handler::<super::compensation_movement::CreditBinding, _>(
            super::compensation_movement::CreditHandler,
        )?;
        setup.handler::<super::compensation_movement::DebitBinding, _>(
            super::compensation_movement::DebitHandler,
        )?;
        setup.handler::<super::principal_status::Binding, _>(super::principal_status::Handler)
    }
}
pub fn install() -> Host {
    install_program(
        validated::<Program>(),
        roster().support(validated::<super::output_program::OutputProgram>()),
    )
}
pub fn install_with_outputs() -> OutputHost {
    install_program(
        validated::<super::output_program::OutputProgram>(),
        roster().support(validated::<Program>()),
    )
}
pub(super) fn install_with_output_limits(
    limits: WorthQueryInMemoryApplicationLimits,
) -> OutputHost {
    install_program_with_seed_and_limits(
        validated::<super::output_program::OutputProgram>(),
        roster().support(validated::<Program>()),
        |_| Ok(()),
        limits,
    )
}
pub fn roster() -> WorthQueryApplicationProgramRoster<'static, NumericSchema> {
    WorthQueryApplicationProgramRoster::new()
        .support(validated::<Successor>())
        .support(validated::<ChangedInverse>())
        .support(validated::<ChangedSource>())
}
pub fn install_program<P>(
    program: ValidatedApplicationProgram<NumericSchema, P>,
    roster: WorthQueryApplicationProgramRoster<'_, NumericSchema>,
) -> WorthQueryProgramApplicationRuntime<NumericSchema, P>
where
    P: ApplicationProgramDefinition<NumericSchema, Contributions = (NumericContribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<NumericSchema>,
{
    install_program_with_seed(program, roster, |_| Ok(()))
}
pub(super) fn install_program_with_seed<P>(
    program: ValidatedApplicationProgram<NumericSchema, P>,
    roster: WorthQueryApplicationProgramRoster<'_, NumericSchema>,
    seed: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> WorthQueryProgramApplicationRuntime<NumericSchema, P>
where
    P: ApplicationProgramDefinition<NumericSchema, Contributions = (NumericContribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<NumericSchema>,
{
    install_program_with_seed_and_limits(program, roster, seed, installation_limits())
}
pub(super) fn install_program_with_seed_and_limits<P>(
    program: ValidatedApplicationProgram<NumericSchema, P>,
    roster: WorthQueryApplicationProgramRoster<'_, NumericSchema>,
    seed: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    limits: WorthQueryInMemoryApplicationLimits,
) -> WorthQueryProgramApplicationRuntime<NumericSchema, P>
where
    P: ApplicationProgramDefinition<NumericSchema, Contributions = (NumericContribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<NumericSchema>,
{
    in_memory_rostered_program(
        program,
        roster,
        NumericSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        limits,
        |_, graph, installed| {
            let binding = installed
                .principal_binding(ActorBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("numeric-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("numeric-reviewer").unwrap(),
                2_u64,
                reviewer_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Number::reference(),
                    WorthQueryApplicationEntityKey::new("numeric").unwrap(),
                )
                .field(Key::at("Key"), "numeric".to_owned())
                .field(Primary::at("Primary"), Counter(5_u64))
                .field(Secondary::at("Secondary"), 7_u64)
                .field(ReadEnabled::at("ReadEnabled"), true)
                .field(Ceiling::at("Ceiling"), 100_u64)
                .field(Calculated::at("Calculated"), 0_u64)
                .field(
                    super::write_admission::Binary::at("Binary"),
                    BinaryCounter(5_u64),
                )
                .field(
                    super::write_admission::Canonical::at("Canonical"),
                    super::write_admission::CanonicalCounter(5),
                )
                .field(
                    super::write_admission::Delegated::at("Delegated"),
                    DelegatedCounter(5_u64),
                )
                .field(
                    super::composite_definition_model::DefinitionRevision::at("DefinitionRevision"),
                    5_u64,
                ),
            )?;
            super::adoption::seed(graph);
            super::causal_inspection::capability::seed(graph);
            seed(graph)
        },
    )
    .unwrap()
}
pub fn edit(host: &Host, value: edit::Value, key: &str) -> WorthQueryApplicationCommitReceipt {
    edit_on(host, host.runtime().current_world(), value, key)
}
pub fn edit_on(
    host: &Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    value: edit::Value,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    if !matches!(
        value,
        edit::Value::Primary(_) | edit::Value::AdjacentPrimary(_)
    ) {
        return edit_other(host, branch, value, key);
    }
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let operation = request
        .mutate(edit::Edit {
            key: "numeric".to_owned(),
            value,
        })
        .without_source()
        .idempotency_owned(key.to_owned())
        .unwrap_or_else(|(_, _, error)| panic!("input capture: {error:?}"))
        .retain_operation::<Program, Action>(host)
        .unwrap_or_else(|(_, error)| panic!("action construction: {error:?}"));
    let completed = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("numeric edit refusal: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("numeric edit terminal: {:?}", terminal.native_outcome())
        }
        WorthQuerySourceOnlyAdvance::Progress(operation) => panic!(
            "numeric edit pending: {:?}, {:?}",
            operation.posture(),
            operation.native_retry_outcome()
        ),
    };
    let (_, evidence) = completed
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, error)| panic!("source close: {error:?}"));
    match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. } => receipt,
        WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    }
}
pub fn edit_other(
    host: &Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    value: edit::Value,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let key = key.to_owned();
    let outcome = request
        .advanced()
        .mutate(other_edit::OtherEdit {
            key: "numeric".to_owned(),
            value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, runtime::ExecutionAllocationPolicy::SystemAllocation)
        .expect("the independent ordinary edit is admitted");
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. }
        | WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => receipt,
        other => panic!("independent edit must publish: {other:?}"),
    }
}
pub fn read<P: ApplicationProgramDefinition<NumericSchema>>(
    host: &WorthQueryProgramApplicationRuntime<NumericSchema, P>,
) -> Vec<query::Row> {
    read_on(host, host.runtime().current_world())
}
pub fn read_on<P: ApplicationProgramDefinition<NumericSchema>>(
    host: &WorthQueryProgramApplicationRuntime<NumericSchema, P>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) -> Vec<query::Row> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .query(query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}

/// Bootstrap issues the ordinary retained projection owner used by decode courts.
pub fn install_with_projection() -> (
    Host,
    WorthQueryApplicationInvariantProjectionAuthority<NumericSchema>,
) {
    let mut authority = None;
    let application = install_program_with_seed(
        validated::<Program>(),
        roster().support(validated::<super::output_program::OutputProgram>()),
        |graph| {
            authority = Some(graph.retain_invariant_projection_authority());
            Ok(())
        },
    );
    (
        application,
        authority.expect("actual bootstrap issued projection authority"),
    )
}

/// Reopen the exact current fixture representation; no migration is inferred.
pub fn restore(checkpoint: WorthQueryApplicationCheckpoint) -> Host {
    in_memory_rostered_program_from_checkpoint(
        validated::<Program>(),
        roster().support(validated::<super::output_program::OutputProgram>()),
        NumericSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        installation_limits(),
        checkpoint,
    )
    .expect("current inline-domain checkpoint reopens")
}
