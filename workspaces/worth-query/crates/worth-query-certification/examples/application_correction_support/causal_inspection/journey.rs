//! Ordinary source, installed arithmetic producer and freshly governed inspection.
use super::super::{
    authentication::*,
    edit::{Edit, Value},
    host,
    output_program::{OutputProgram, Root},
    schema::*,
};
use super::{capability, query};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::*,
    application_inspection::*,
    primary_graph::{
        WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationCommitReceipt,
        WorthQueryConditionalSignalDecision, WorthQueryPrincipalResolutionMode,
    },
    runtime::ExecutionAllocationPolicy,
};
type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    NumericSchema,
    capability::Inspect,
    capability::Operation,
    capability::Input,
>;

pub fn connected_source_and_output() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let baseline = application
        .application_query_basis_observer()
        .observe()
        .active();
    let (before, initial) = read(&application, &principal, &scope, None);
    assert_eq!((initial.input, initial.calculated), (5, 0));
    let (source, producer, observation) = perform(&application, 11, "causal-source-eleven");
    let (after, final_row) = read(&application, &principal, &scope, Some(&observation));
    assert_eq!((final_row.input, final_row.calculated), (11, 23));
    drop(observation);
    let expected_source = source.commit_reference().clone();
    let expected_producer = producer.commit_reference().clone();
    let fresh = application.runtime().request(&principal, &scope);
    let report = fresh
        .inspect::<query::Binding>(&after)
        .why_changed(&before)
        .caused_by(&source)
        .reference_only()
        .execute_with_diagnostic(access(&application, &principal, &scope))
        .unwrap();
    assert_eq!(report.reason(), CausalInspectionReason::ChangedResult);
    let connection = report
        .producer_connection()
        .expect("both real field/revision joins were established");
    assert_eq!(connection.source_commit(), &expected_source);
    assert_eq!(connection.producer_commit(), &expected_producer);
    assert_eq!(connection.output_role(), "calculated-number");
    assert_eq!(
        connection.readiness_decision(),
        WorthQueryConditionalSignalDecision::Eligible
    );
    drop((source, producer));
    assert_eq!(
        report.producer_connection().unwrap().source_commit(),
        &expected_source
    );
    assert_eq!(
        application
            .application_query_basis_observer()
            .observe()
            .active(),
        baseline
    );
    report.close();
    before.close();
    after.close();
    assert_eq!(
        application
            .application_query_basis_observer()
            .observe()
            .active(),
        baseline
    );
}

pub fn equal_values_and_head_reads_cannot_supply_a_cause() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (before, _) = read(&application, &principal, &scope, None);
    let (earlier, _, _) = perform(&application, 11, "causal-old-eleven");
    // A no-op write need not create a Native field revision. Change away and
    // back so the equal-looking old source is genuinely another occurrence.
    let _ = perform(&application, 7, "causal-intervening-seven");
    let (source, _, observation) = perform(&application, 11, "causal-rewritten-eleven");
    assert_ne!(earlier.commit_reference(), source.commit_reference());
    let (after, row) = read(&application, &principal, &scope, Some(&observation));
    assert_eq!(row.input, 11);
    let request = application.runtime().request(&principal, &scope);
    let refused = request
        .inspect::<query::Binding>(&after)
        .why_changed(&before)
        .caused_by(&earlier)
        .reference_only()
        .execute_with_diagnostic(access(&application, &principal, &scope))
        .err()
        .expect("an equal value from another write is not the actual input occurrence");
    assert_eq!(
        refused.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
    );
    let (head, same_row) = read(&application, &principal, &scope, None);
    assert_eq!(same_row, row);
    let refused = request
        .inspect::<query::Binding>(&head)
        .why_changed(&before)
        .caused_by(&source)
        .reference_only()
        .execute_with_diagnostic(access(&application, &principal, &scope))
        .err()
        .expect("a same-looking head query cannot infer its supplier");
    assert_eq!(
        refused.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::ProducerUnavailable)
    );
    let foreign = host::install_with_outputs();
    let (foreign_source, _, _) = perform(&foreign, 11, "causal-foreign-eleven");
    let refused = request
        .inspect::<query::Binding>(&after)
        .why_changed(&before)
        .caused_by(&foreign_source)
        .reference_only()
        .execute_with_diagnostic(access(&application, &principal, &scope))
        .err()
        .expect("a genuine foreign source has no local causal authority");
    assert_eq!(
        refused.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::ForeignSource)
    );
}

pub fn current_admission_precedes_source_lookup() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (before, _) = read(&application, &principal, &scope, None);
    let (source, _, observation) = perform(&application, 11, "causal-revocation-source");
    let (after, _) = read(&application, &principal, &scope, Some(&observation));
    let admitted = access(&application, &principal, &scope);
    let request = application.runtime().request(&principal, &scope);
    request
        .advanced()
        .mutate(super::super::principal_status::DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&"causal-disable-principal".to_owned())
        .execute()
        .unwrap();
    let refusal = request
        .inspect::<query::Binding>(&after)
        .why_changed(&before)
        .caused_by(&source)
        .reference_only()
        .execute_with_diagnostic(admitted)
        .err()
        .expect("old payloads and capabilities cannot replace current READ");
    assert!(
        refusal.producer_connection_refusal().is_none(),
        "fresh authority refused before source evidence lookup"
    );
}

fn read(
    application: &host::OutputHost,
    principal: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    observation: Option<&WorthQueryApplicationReadObservation>,
) -> (WorthQueryApplicationQueryObservation, query::Row) {
    let request = application.runtime().request(principal, scope);
    let intent = query::Read {
        key: "numeric".to_owned(),
    };
    let result = match observation {
        Some(observation) => request
            .at(observation)
            .query(intent)
            .retain_projection_for_inspection()
            .execute_governed(
                capability::Inspect::reference(),
                capability::Operation::reference(),
                capability::Input,
            ),
        None => request
            .query(intent)
            .retain_projection_for_inspection()
            .execute_governed(
                capability::Inspect::reference(),
                capability::Operation::reference(),
                capability::Input,
            ),
    }
    .unwrap();
    assert_eq!(result.rows().len(), 1);
    (
        result.receipt().inspect().observation().unwrap().clone(),
        result.rows()[0].clone(),
    )
}

fn perform(
    application: &host::OutputHost,
    value: u64,
    key: &str,
) -> (
    WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationReadObservation,
) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let key = key.to_owned();
    let source = request
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(value),
        })
        .without_source()
        .idempotency(&key)
        .execute_performed::<OutputProgram, Root>(
            application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(source) = source else {
        panic!("ordinary source must perform");
    };
    let receipt = source.receipt().clone();
    let mut started = source
        .start_required_outputs(
            application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|_| panic!("actual required output starts"));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(application, &request)
        .unwrap()
    else {
        panic!("the installed arithmetic producer settles");
    };
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    (
        receipt,
        settled.root_receipt().unwrap().clone(),
        settled.root_observation().retain(),
    )
}

fn access(
    application: &host::OutputHost,
    external: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
) -> Access {
    application
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let binding = application
                .installed_schema()
                .principal_binding(ActorBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let capability = application
                .installed_schema()
                .capability(
                    capability::Inspect::reference(),
                    capability::Operation::reference(),
                )
                .unwrap();
            selected
                .admit_capability_access(&phase, &principal, &capability, capability::Input, scope)
                .unwrap()
        })
        .unwrap()
}
