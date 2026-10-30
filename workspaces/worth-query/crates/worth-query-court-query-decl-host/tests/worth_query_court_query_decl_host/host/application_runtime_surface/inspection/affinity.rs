//! Genuine binding and runtime affinity before original-observation disclosure.
use super::super::{
    comparison_model::GovernedAccountReadBinding,
    inspection_authentication::authenticate,
    inspection_model::{AccountRead, AccountReadBinding},
    installed_application,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::WorthQueryApplicationRequestExt,
    causal_inspection::WorthQueryApplicationInspectionStopKind,
};

#[test]
fn original_observation_refuses_another_genuine_binding_and_runtime() {
    let application = installed_application();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(application.installed_schema(), &scope);
    let request = application.request(&principal, &scope);
    let result = request
        .query(AccountRead("account-001".into()))
        .execute()
        .unwrap();
    assert_eq!(result.rows(), &["account-001".to_owned()]);
    let original = result.receipt().inspect().observation().unwrap();
    let wrong_binding = request
        .inspect::<GovernedAccountReadBinding>(original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert_eq!(
        wrong_binding.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(wrong_binding.producer_output_refusal().is_none());
    drop(wrong_binding);

    let foreign = installed_application();
    // Authenticate with the receiver so a foreign external credential cannot
    // substitute for the original observation's runtime-affinity refusal.
    let foreign_principal = authenticate(foreign.installed_schema(), &scope);
    let foreign_request = foreign.request(&foreign_principal, &scope);
    let denied = foreign_request
        .inspect::<AccountReadBinding>(original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .unwrap();
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denied.producer_output_refusal().is_none());
    drop(denied);
    request
        .inspect::<AccountReadBinding>(original)
        .why_observed()
        .reference_only()
        .execute()
        .unwrap()
        .close();
    let foreign_result = foreign_request
        .query(AccountRead("account-001".into()))
        .execute()
        .unwrap();
    foreign_request
        .inspect::<AccountReadBinding>(foreign_result.receipt().inspect().observation().unwrap())
        .why_observed()
        .reference_only()
        .execute()
        .unwrap()
        .close();
}
