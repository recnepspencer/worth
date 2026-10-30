//! Genuine ordinary receipt journey and supported Decl/Host entrances.
#[path = "inspection/rich.rs"]
mod rich;
use worth_query_decl::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::ApplicationSchema,
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequest,
    causal_inspection::{
        WorthQueryApplicationInspectionDenial, WorthQueryApplicationQueryObservation,
        WorthQueryObservedApplicationInspection,
    },
    primary_graph::WorthQueryAdmittedApplicationCapabilityAccess,
};

#[allow(dead_code)]
fn ordinary<'o, Schema, Binding>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    original: &'o WorthQueryApplicationQueryObservation,
) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    request
        .inspect::<Binding>(original)
        .why_observed()
        .reference_only()
        .execute()
}

#[allow(dead_code)]
fn governed<'o, Schema, Binding, Capability, Operation, Input>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    original: &'o WorthQueryApplicationQueryObservation,
    capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    Input: ApplicationCapabilityRequest<
        Schema,
        Capability,
        Scope = <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >,
{
    request
        .inspect::<Binding>(original)
        .why_observed()
        .reference_only()
        .execute_governed(capability)
}

#[test]
fn ordinary_published_receipt_enters_fresh_host_inspection() {
    use super::{
        inspection_authentication::authenticate,
        inspection_model::{AccountRead, AccountReadBinding},
    };
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        application_entry::WorthQueryApplicationRequestExt,
        causal_inspection::{
            CausalInspectionReason, CausalObservationMissingReferencePosture,
            WorthQueryApplicationInspectionStopKind,
        },
    };
    let application = super::installed_application();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    let external = authenticate(application.installed_schema(), &scope);
    let request = application.request(&external, &scope);
    let registry = application.application_query_basis_observer();
    let before = registry.observe().active();
    let result = request
        .query(AccountRead("account-001".into()))
        .execute()
        .unwrap();
    assert_eq!(result.rows(), &["account-001".to_string()]);
    assert!(result.receipt().inspect().terminal_resources_released());
    // Borrowing through a temporary inspection view preserves the receipt loan.
    let original = result
        .receipt()
        .inspect()
        .observation()
        .expect("real bound completion retains its observation");
    let explanation =
        ordinary::<super::HostIdentitySchema, AccountReadBinding>(&request, original).unwrap();
    assert!(std::ptr::eq(explanation.observation(), original));
    assert_eq!(explanation.reason(), CausalInspectionReason::ObservedResult);
    assert_eq!(
        explanation.lower_evidence_posture(),
        CausalObservationMissingReferencePosture::MissingOptional
    );
    assert!(!explanation.observation_identity().is_empty());
    assert!(!explanation.report_identity().is_empty());
    assert_eq!(registry.observe().active(), before);
    explanation.close();
    cancellation.cancel();
    let denial = match ordinary::<super::HostIdentitySchema, AccountReadBinding>(&request, original)
    {
        Ok(_) => panic!("the reused request must freshly observe cancellation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(denial);
    drop(result);
    assert_eq!(registry.observe().active(), before);
}

#[test]
fn retained_public_projection_survives_result_and_sibling_close_through_two_facades() {
    use super::{
        inspection_authentication::authenticate,
        inspection_model::{AccountRead, AccountReadBinding},
    };
    use worth_query_host::facade::{
        admission::authenticated_principal::{
            WorthQueryCancellationSource, WorthQueryRequestScope,
        },
        application_entry::WorthQueryApplicationRequestExt,
        causal_inspection::{
            WorthQueryInspectionRetainedEvidence as Retained, WorthQueryInspectionSupplierEvidence,
        },
    };
    let application = super::installed_application();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(application.installed_schema(), &scope);
    let request = application.request(&principal, &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let result = request
        .query(AccountRead("account-001".into()))
        .retain_projection_for_inspection()
        .execute()
        .unwrap();
    assert_eq!(result.rows(), &["account-001".to_owned()]);
    let original = result.receipt().inspect().observation().unwrap().clone();
    let sibling = original.clone();
    drop(result);
    original.close();
    let report =
        ordinary::<super::HostIdentitySchema, AccountReadBinding>(&request, &sibling).unwrap();
    let availability = report.evidence_availability();
    assert_eq!(availability.projection(), Retained::Retained);
    assert_eq!(
        availability.supplier(),
        WorthQueryInspectionSupplierEvidence::NotRetained
    );
    assert_eq!(availability.native_observation(), Retained::NotRetained);
    assert_eq!(availability.conditional_history(), None);
    assert_eq!(readers.observe().active(), baseline);
    report.close();
    sibling.close();
    assert_eq!(readers.observe().active(), baseline);
}
