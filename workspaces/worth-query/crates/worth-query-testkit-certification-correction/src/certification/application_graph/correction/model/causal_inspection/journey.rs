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
    application_installation::WorthQueryProgramApplicationRuntime,
    causal_inspection::*,
    declaration::{
        application_operation::ApplicationMutationBinding,
        application_program::ApplicationProgramDefinition,
    },
    primary_graph::{
        WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationRequiredOutputSource, WorthQueryConditionalSignalDecision,
        WorthQueryPrincipalResolutionMode,
    },
    runtime::ExecutionAllocationPolicy,
};
mod availability;
mod availability_lifecycle;
pub mod paired;
mod retained_query;
pub use availability::original_evidence_availability_requires_fresh_metadata;
pub use availability_lifecycle::availability_keeps_sibling_custody_and_releases_final_funding;
type Access = WorthQueryAdmittedApplicationCapabilityAccess<
    NumericSchema,
    capability::Inspect,
    capability::InspectCausalEvidence,
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
        .execute_governed(access(&application, &principal, &scope),access(&application, &principal, &scope))
        .unwrap_or_else(|denial| {
            panic!(
                "fresh connected inspection must admit: {denial:?}; source={expected_source:?}; producer={expected_producer:?}; connection={:?}; resource={:?}; canonical={:?}",
                denial.producer_connection_refusal(),
                denial.resource_refusal(),
                denial.canonical_refusal(),
            )
        });
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
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
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
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
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
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
        .err()
        .expect("a genuine foreign source has no local causal authority");
    assert_eq!(
        refused.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::ForeignSource)
    );
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &scope);
    foreign
        .runtime()
        .request(&foreign_principal, &scope)
        .inspect_source::<super::SourceDisclosure>(&foreign_source)
        .why_performed()
        .reference_only()
        .execute_governed(access(&foreign, &foreign_principal, &scope))
        .unwrap()
        .close();
}

pub fn current_admission_precedes_source_lookup() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (before, _) = read(&application, &principal, &scope, None);
    let (source, _, observation) = perform(&application, 11, "causal-revocation-source");
    let (after, _) = read(&application, &principal, &scope, Some(&observation));
    let admitted = access(&application, &principal, &scope);
    let metadata = access(&application, &principal, &scope);
    let request = application.runtime().request(&principal, &scope);
    request
        .advanced()
        .mutate(super::super::principal_status::DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&"causal-disable-principal".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let refusal = request
        .inspect::<query::Binding>(&after)
        .why_changed(&before)
        .caused_by(&source)
        .reference_only()
        .execute_governed(metadata, admitted)
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
    if let Some(observation) = observation {
        return retained_query::read(application, principal, scope, observation);
    }
    let request = application.runtime().request(principal, scope);
    let intent = query::Read {
        key: "numeric".to_owned(),
    };
    let result = request
        .query(intent)
        .retain_projection_for_inspection()
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    (
        result.receipt().inspect().observation().unwrap().clone(),
        result.rows()[0].clone(),
    )
}

pub(super) fn perform(
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
    let source = match source {
        WorthQueryApplicationPerformedMutationOutcome::Performed(source) => source,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("ordinary source must perform; actual native outcome: {outcome:?}")
        }
        _ => panic!("ordinary source must perform; required-output custody was not ready"),
    };
    let (source, producer, read) = settle_source(application, &request, source, 1);
    (
        source,
        producer.expect("one actual performed producer"),
        read,
    )
}

pub(super) fn settle_source<Binding>(
    application: &host::OutputHost,
    request: &WorthQueryApplicationRequest<'_, '_, '_, NumericSchema>,
    source: WorthQueryPerformedApplicationMutation<NumericSchema, Binding, OutputProgram, Root>,
    expected_contacts: usize,
) -> (
    WorthQueryApplicationCommitReceipt,
    Option<WorthQueryApplicationCommitReceipt>,
    WorthQueryApplicationReadObservation,
)
where
    Binding: ApplicationMutationBinding<NumericSchema>
        + WorthQueryApplicationRequiredOutputSource<
            NumericSchema,
            super::super::output_program::Connection,
        >,
{
    let receipt = source.receipt().clone();
    let mut started = source
        .start_required_outputs(
            application,
            request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|_| panic!("actual required output starts"));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(application, request)
        .unwrap()
    else {
        panic!("the installed arithmetic producer settles");
    };
    assert_eq!(
        settled.root_producer_contacts_in_this_demand(),
        expected_contacts
    );
    (
        receipt,
        settled.root_receipt().cloned(),
        settled.root_observation().retain(),
    )
}

pub(in super::super) fn access<P: ApplicationProgramDefinition<NumericSchema>>(
    application: &WorthQueryProgramApplicationRuntime<NumericSchema, P>,
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
                    capability::InspectCausalEvidence::reference(),
                )
                .unwrap();
            selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &capability,
                    capability::Input::at("numeric"),
                    scope,
                )
                .unwrap()
        })
        .unwrap()
}
