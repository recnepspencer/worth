//! Install, edit, read and close through the public application owners.
use super::{authentication::*, edit, other_edit, program::*, query, schema::*};
use worth_query_host::facade::{
    application_contribution::*,
    application_entry::*,
    application_installation::*,
    declaration::{application_program::*, authentication::*},
    primary_graph::*,
    runtime,
};

// The application graph court owns and lints these resources; the example uses a part.
#[allow(dead_code)]
#[path = "../../tests/application_graph/document_retention_model/host/world_resources.rs"]
mod world_resources;
pub type Host = WorthQueryProgramApplicationRuntime<NumericSchema, Program>;
pub type OutputHost =
    WorthQueryProgramApplicationRuntime<NumericSchema, super::output_program::OutputProgram>;
impl WorthQueryApplicationContribution<NumericSchema> for NumericContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<super::calculation::Producer>()?;
        contracts.conditional::<super::calculation_readiness::Readiness>()?;
        contracts.conditional::<super::composite_definition_runtime::Conditional>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, NumericSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<super::calculation::Binding, _>(super::calculation::Handler)?;
        setup.producer::<super::calculation::Producer>(super::calculation::Provider)?;
        setup.conditional::<super::calculation_readiness::Readiness>(())?;
        setup.conditional::<super::composite_definition_runtime::Conditional>(())?;
        setup.handler::<super::composite_definition_model::Binding, _>(
            super::composite_definition_handler::Handler,
        )?;
        setup.handler::<edit::EditBinding, _>(edit::Handler)?;
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
fn roster() -> WorthQueryApplicationProgramRoster<'static, NumericSchema> {
    WorthQueryApplicationProgramRoster::new()
        .support(validated::<Successor>())
        .support(validated::<ChangedInverse>())
        .support(validated::<ChangedSource>())
}
fn install_program<P>(
    program: ValidatedApplicationProgram<NumericSchema, P>,
    roster: WorthQueryApplicationProgramRoster<'_, NumericSchema>,
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
        WorthQueryInMemoryApplicationLimits::new(
            world_resources::world_resources(256, 4 * 1024 * 1024, 256),
            runtime::WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(128).unwrap(),
            ),
            SignalConditionalEvaluationBudget::development(),
        ),
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
                .field(Key::reference(), "numeric".to_owned())
                .field(Primary::reference(), 5_u64)
                .field(Secondary::reference(), 7_u64)
                .field(ReadEnabled::reference(), true)
                .field(Ceiling::reference(), 100_u64)
                .field(Calculated::reference(), 0_u64),
            )?;
            super::adoption::seed(graph);
            Ok(())
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
    if !matches!(value, edit::Value::Primary(_)) {
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
fn edit_other(
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
