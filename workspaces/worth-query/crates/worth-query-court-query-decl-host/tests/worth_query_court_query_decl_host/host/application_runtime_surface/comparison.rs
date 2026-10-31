//! Public Decl/Host journey from two real published query receipts.
use super::{
    comparison_model::{GovernedAccountRead, GovernedAccountReadBinding},
    inspection_authentication::authenticate,
    inspection_capability::{InspectAccountCapability, InspectAccountOperation},
    IdentityBinding,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::WorthQueryApplicationRequestExt,
    causal_inspection::CausalInspectionReason,
    primary_graph::WorthQueryPrincipalResolutionMode,
};
#[test]
fn governed_published_receipts_enter_fresh_paired_host_inspection() {
    let application = super::installed_application();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    let external = authenticate(application.installed_schema(), &scope);
    let request = application.request(&external, &scope);
    let registry = application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let read = || {
        request
            .query(GovernedAccountRead("account-001".into()))
            .retain_projection_for_inspection()
            .execute_governed(
                InspectAccountCapability::reference(),
                InspectAccountOperation::reference(),
                "account-001".to_owned(),
            )
            .unwrap()
    };
    let before = read();
    let after = read();
    assert_eq!(before.rows(), &["account-001".to_owned()]);
    assert_eq!(after.rows(), before.rows());
    let before_original = before.receipt().inspect().observation().unwrap();
    let after_original = after.receipt().inspect().observation().unwrap();
    assert!(!std::ptr::eq(before_original, after_original));
    let diagnostic = application
        .with_host_advancement(|phase| {
            let selected = application
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let binding = application
                .installed_schema()
                .principal_binding(IdentityBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let capability = application
                .installed_schema()
                .capability(
                    InspectAccountCapability::reference(),
                    InspectAccountOperation::reference(),
                )
                .unwrap();
            selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &capability,
                    "account-001".to_owned(),
                    &scope,
                )
                .unwrap()
        })
        .unwrap();
    let comparison = request
        .inspect::<GovernedAccountReadBinding>(after_original)
        .why_changed(before_original)
        .reference_only()
        .execute_with_diagnostic(diagnostic)
        .unwrap();
    assert_eq!(comparison.reason(), CausalInspectionReason::UnchangedResult);
    assert!(std::ptr::eq(comparison.before(), before_original));
    assert!(std::ptr::eq(comparison.after(), after_original));
    assert_ne!(
        comparison.before_report_identity(),
        comparison.after_report_identity()
    );
    assert_ne!(
        comparison.report_identity(),
        comparison.before_report_identity()
    );
    assert_ne!(
        comparison.report_identity(),
        comparison.after_report_identity()
    );
    assert_eq!(registry.observe().active(), baseline);
    comparison.close();
    drop(after);
    drop(before);
    assert_eq!(registry.observe().active(), baseline);
}
