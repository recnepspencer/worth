//! Ordinary Calculated-only reports require both independently installed admissions.
use super::super::{authentication::*, host, schema::*};
use super::{capability, journey, output_query, query, SourceDisclosure, SourceOnlyDisclosure};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::*,
    causal_inspection::*,
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
};
mod lifecycle;
mod parameters;
pub(super) mod retained_query;
pub use lifecycle::current_admission_cancellation_and_expiry_precede_connection_discovery;
pub use parameters::no_match_parameters_keep_original_output_evidence_unavailable;

fn metadata<Binding: ApplicationQueryBinding<NumericSchema>>(
    name: &'static str,
) -> ApplicationQueryInspectionDefinition<NumericSchema, Binding> {
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(
            ApplicationQueryDisclosureContract::governed_by(name, capability::Inspect::reference()),
            |contract, facet| {
                contract.disclose_inspection_metadata_by(
                    ApplicationQueryInspectionMetadataRef::<NumericSchema, Binding>::new(facet),
                    capability::text("calculation"),
                    ApplicationQueryInfluenceContract::permit([facet.influence()]),
                )
            },
        );
    ApplicationQueryInspectionDefinition::new(contract).unwrap()
}
pub(super) fn source_metadata(
) -> ApplicationQueryInspectionDefinition<NumericSchema, query::Binding> {
    metadata::<query::Binding>("numeric-original-source-metadata")
}
pub(super) fn output_metadata(
) -> ApplicationQueryInspectionDefinition<NumericSchema, output_query::Binding> {
    metadata::<output_query::Binding>("numeric-observed-output-metadata")
}

pub fn calculated_only_connects_original_source_under_independent_governance() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let baseline = application
        .application_query_basis_observer()
        .observe()
        .active();
    let (source, producer, read) =
        journey::perform(&application, 11, "independent-source-positive");
    let expected_source = source.commit_reference().clone();
    let expected_producer = producer.commit_reference().clone();
    let (after, row) = retained_query::read(&application, &principal, &scope, &read);
    assert_eq!(row.calculated, 23);
    drop(read);
    use worth_query_host::facade::runtime::{
        CancellationToken, DeterminismContract, ExecutionAllocationPolicy, ExecutionAuthority,
        ExecutionPosture, ExecutionRequestPolicy, LeaseRequest,
    };
    let authority = ExecutionAuthority::serial();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(32 * 1024 * 1024),
                    Some(100_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let before_memory = lease.charged_memory_bytes();
    let fresh = application.runtime().request(&principal, &scope);
    let report = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
        )
        .unwrap();
    assert_eq!(report.reason(), CausalInspectionReason::ObservedResult);
    assert_eq!(
        report.producer_connection().source_commit(),
        &expected_source
    );
    assert_eq!(
        report.producer_connection().producer_commit(),
        &expected_producer
    );
    assert_eq!(
        report.producer_connection().output_role(),
        "calculated-number"
    );
    assert!(!report.report_identity().is_empty());
    assert!(lease.charged_memory_bytes() > before_memory);
    drop((source, producer));
    assert!(lease.charged_memory_bytes() > before_memory);
    assert_eq!(
        report.producer_connection().source_commit(),
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
    assert_eq!(lease.charged_memory_bytes(), before_memory);
    after.close();
    assert_eq!(
        application
            .application_query_basis_observer()
            .observe()
            .active(),
        baseline
    );
}

pub fn neither_binding_can_lend_metadata_diagnostics_or_ownership() {
    let application = host::install_with_outputs();
    let foreign = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let foreign_scope = request_scope();
    let foreign_principal = authenticate_operator(foreign.installed_schema(), &foreign_scope);
    let (source, _, read) = journey::perform(&application, 11, "independent-admission-controls");
    let (after, _) = retained_query::read(&application, &principal, &scope, &read);
    drop(read);
    let fresh = application.runtime().request(&principal, &scope);
    for role in 0..4 {
        let access = |position| {
            if role == position {
                journey::access(&foreign, &foreign_principal, &foreign_scope)
            } else {
                journey::access(&application, &principal, &scope)
            }
        };
        let denial = fresh
            .inspect::<output_query::Binding>(&after)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_governed(access(0), access(1), access(2), access(3))
            .err()
            .expect("each actual role refuses foreign capability ownership");
        assert!(
            denial.producer_connection_refusal().is_none(),
            "admission must refuse before protected cause lookup"
        );
    }
    assert!(
        fresh
            .inspect::<output_query::Binding>(&after)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_with_diagnostics(
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope)
            )
            .is_err(),
        "diagnostic grants cannot replace either metadata contract"
    );
    assert!(
        fresh
            .inspect::<output_query::Binding>(&after)
            .why_observed()
            .caused_by::<SourceOnlyDisclosure>(&source)
            .reference_only()
            .execute_governed(
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope)
            )
            .is_err(),
        "a different original declaration cannot reinterpret the source binding"
    );
    let (foreign_source, _, _) = journey::perform(&foreign, 11, "independent-foreign-source");
    assert!(fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&foreign_source)
        .reference_only()
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope)
        )
        .is_err());
    // Both source and query own complete binding references, including parameters.
    // The caller has no override parameter capsule that could replace either original.
    assert!(
        fresh
            .inspect::<query::Binding>(&after)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_governed(
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope),
                journey::access(&application, &principal, &scope)
            )
            .is_err(),
        "another complete binding cannot consume the output observation"
    );
    after.close();
}

/// Same value after an intervening source cannot replace the actual original revision.
pub fn equal_source_rewrites_do_not_connect_to_the_later_producer() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (earlier, _, _) = journey::perform(&application, 11, "independent-earlier-eleven");
    let _ = journey::perform(&application, 7, "independent-intervening-seven");
    let (actual, producer, read) = journey::perform(&application, 11, "independent-current-eleven");
    assert_ne!(earlier.commit_reference(), actual.commit_reference());
    let (after, row) = retained_query::read(&application, &principal, &scope, &read);
    assert_eq!(row.calculated, 23);
    drop(read);
    let fresh = application.runtime().request(&principal, &scope);
    let denial = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&earlier)
        .reference_only()
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
        )
        .err()
        .expect("equal values cannot substitute a source-created revision");
    assert_eq!(
        denial.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
    );
    let report = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&actual)
        .reference_only()
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
        )
        .unwrap();
    assert_eq!(
        report.producer_connection().producer_commit(),
        producer.commit_reference()
    );
    report.close();
    after.close();
}
