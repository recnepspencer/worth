//! Distinct original-receipt admission and later paired connection affinity.
use super::super::super::{causal_inspection::capability, required_output_courts};
use super::super::query;
use super::*;
use worth_query_host::facade::product::WorthQueryProductBranch;

pub(super) fn observe(
    application: &host::OutputHost,
) -> (WorthQueryApplicationQueryObservation, query::Row) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let result = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .retain_projection_for_inspection()
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input,
        )
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    (
        result.receipt().inspect().observation().unwrap().clone(),
        result.rows()[0].clone(),
    )
}

pub(super) fn refuse_foreign_host(
    application: &host::OutputHost,
    after: &WorthQueryApplicationQueryObservation,
) {
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let head = required_output_courts::head(application);
    let foreign = host::install_with_outputs();
    let _ = perform(
        &foreign,
        Action::Set {
            bias: 7,
            sibling: 70,
        },
        "foreign-aspect-present",
        1,
    );
    let (foreign_source, producer, read) =
        perform(&foreign, Action::Remove, "foreign-aspect-remove", 1);
    assert!(producer.is_some());
    drop(read);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let denial = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<output_query::Binding>(after)
        .why_observed()
        .caused_by::<AspectDisclosure>(&foreign_source)
        .reference_only()
        .execute_governed(
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
        )
        .err()
        .expect("a genuine foreign source refuses before connection admission");
    // The public posture intentionally exposes no protected original-affinity cause.
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denial.producer_connection_refusal().is_none());
    assert!(denial.resource_refusal().is_none());
    drop((denial, foreign_source, producer));
    assert_eq!(required_output_courts::head(application), head);
    assert_eq!(readers.observe().active(), baseline);
}

pub(super) fn refuse_sibling_source(
    application: &host::OutputHost,
    before: &WorthQueryApplicationQueryObservation,
) {
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let head = required_output_courts::head(application);
    let branch = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .branches()
                .fork(application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let source = perform_sibling(application, branch);
    let (after, row) = observe(application);
    assert_eq!(
        (row.input, row.bias, row.sibling, row.calculated),
        (5, None, None, 11)
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let denial = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&after)
        .why_changed(before)
        .caused_by(&source)
        .reference_only()
        .execute_governed(
            journey::access(application, &principal, &scope),
            journey::access(application, &principal, &scope),
        )
        .err()
        .expect("same-runtime sibling source reaches the later connection affinity guard");
    assert_eq!(
        denial.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::ForeignSource)
    );
    assert!(denial.resource_refusal().is_none());
    assert_eq!(required_output_courts::head(application), head);
    drop((denial, source));
    after.close();
    let closed = application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .on_branch(branch)
                .close(&phase)
                .expect("no sibling observation or source custody survives the negative")
        })
        .unwrap();
    assert!(closed.is_complete());
    assert!(application
        .runtime()
        .branches()
        .pending_cleanup()
        .is_empty());
    let (final_observation, row) = observe(application);
    assert_eq!(
        (row.input, row.bias, row.sibling, row.calculated),
        (5, None, None, 11)
    );
    final_observation.close();
    assert_eq!(required_output_courts::head(application), head);
    assert_eq!(readers.observe().active(), baseline);
}

fn perform_sibling(
    application: &host::OutputHost,
    branch: WorthQueryProductBranch,
) -> WorthQueryApplicationCommitReceipt {
    assert_ne!(
        branch,
        application.current_world(),
        "the source is ordinarily authored on a real sibling"
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    let key = "aspect-affinity-sibling".to_owned();
    let source = request
        .advanced()
        .mutate(AspectEdit {
            key: "numeric".into(),
            action: Action::Set {
                bias: 9,
                sibling: 90,
            },
        })
        .without_source()
        .idempotency(&key)
        .execute_performed::<OutputProgram, Root>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(source) = source else {
        panic!("the real sibling source must perform with required-output custody")
    };
    let receipt = source.receipt().clone();
    let mut started = source
        .start_required_outputs(
            application,
            &request,
            WorthQueryOutputDemandControls::host_policy(),
        )
        .unwrap_or_else(|failure| panic!("sibling required outputs start: {:?}", failure.denial()));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(application, &request)
        .unwrap()
    else {
        panic!("the real sibling producer settles")
    };
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    assert!(settled.root_receipt().is_some());
    assert_eq!(receipt.product_branch(), branch);
    receipt
}
