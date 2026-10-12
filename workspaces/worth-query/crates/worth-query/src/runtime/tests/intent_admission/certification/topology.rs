use super::*;
use crate::intent_admission::{
    INTENT_ADMISSION_CERTIFICATION_CHILD_MODULES, INTENT_ADMISSION_CERTIFICATION_EXPORTED_SURFACE,
    INTENT_ADMISSION_CERTIFICATION_MODULE_ROOT, INTENT_ADMISSION_DECISIONS_CHILD_MODULES,
    INTENT_ADMISSION_DECISIONS_EXPORTED_SURFACE, INTENT_ADMISSION_DECISIONS_MODULE_ROOT,
    INTENT_ADMISSION_DX_CHILD_MODULES, INTENT_ADMISSION_DX_EXPORTED_SURFACE,
    INTENT_ADMISSION_DX_MODULE_ROOT, INTENT_ADMISSION_ELIGIBILITY_CHILD_MODULES,
    INTENT_ADMISSION_ELIGIBILITY_EXPORTED_SURFACE, INTENT_ADMISSION_ELIGIBILITY_MODULE_ROOT,
    INTENT_ADMISSION_FAMILIES_CHILD_MODULES, INTENT_ADMISSION_FAMILIES_EXPORTED_SURFACE,
    INTENT_ADMISSION_FAMILIES_MODULE_ROOT, INTENT_ADMISSION_HANDOFFS_CHILD_MODULES,
    INTENT_ADMISSION_HANDOFFS_EXPORTED_SURFACE, INTENT_ADMISSION_HANDOFFS_MODULE_ROOT,
    INTENT_ADMISSION_SUPPORT_CHILD_MODULES, INTENT_ADMISSION_SUPPORT_EXPORTED_SURFACE,
    INTENT_ADMISSION_SUPPORT_MODULE_ROOT, INTENT_ADMISSION_TRACE_CHILD_MODULES,
    INTENT_ADMISSION_TRACE_EXPORTED_SURFACE, INTENT_ADMISSION_TRACE_MODULE_ROOT,
};

#[test]
fn intent_admission_topology_audit_freezes_real_subdomains_and_binding_ownership() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_intent_admission(request_execution);
    let rows = bundle.topology_audit().rows();

    let expected_rows = [
        (
            WorthQueryIntentAdmissionTopologyDomain::Families,
            INTENT_ADMISSION_FAMILIES_MODULE_ROOT,
            INTENT_ADMISSION_FAMILIES_CHILD_MODULES,
            INTENT_ADMISSION_FAMILIES_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Eligibility,
            INTENT_ADMISSION_ELIGIBILITY_MODULE_ROOT,
            INTENT_ADMISSION_ELIGIBILITY_CHILD_MODULES,
            INTENT_ADMISSION_ELIGIBILITY_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Decisions,
            INTENT_ADMISSION_DECISIONS_MODULE_ROOT,
            INTENT_ADMISSION_DECISIONS_CHILD_MODULES,
            INTENT_ADMISSION_DECISIONS_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Handoffs,
            INTENT_ADMISSION_HANDOFFS_MODULE_ROOT,
            INTENT_ADMISSION_HANDOFFS_CHILD_MODULES,
            INTENT_ADMISSION_HANDOFFS_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Trace,
            INTENT_ADMISSION_TRACE_MODULE_ROOT,
            INTENT_ADMISSION_TRACE_CHILD_MODULES,
            INTENT_ADMISSION_TRACE_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Dx,
            INTENT_ADMISSION_DX_MODULE_ROOT,
            INTENT_ADMISSION_DX_CHILD_MODULES,
            INTENT_ADMISSION_DX_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Support,
            INTENT_ADMISSION_SUPPORT_MODULE_ROOT,
            INTENT_ADMISSION_SUPPORT_CHILD_MODULES,
            INTENT_ADMISSION_SUPPORT_EXPORTED_SURFACE,
        ),
        (
            WorthQueryIntentAdmissionTopologyDomain::Certification,
            INTENT_ADMISSION_CERTIFICATION_MODULE_ROOT,
            INTENT_ADMISSION_CERTIFICATION_CHILD_MODULES,
            INTENT_ADMISSION_CERTIFICATION_EXPORTED_SURFACE,
        ),
    ];

    assert_eq!(rows.len(), expected_rows.len());
    for (row, (domain, module_root, child_modules, exported_surface)) in
        rows.iter().zip(expected_rows.iter())
    {
        assert_eq!(row.domain(), *domain);
        assert_eq!(row.module_root(), *module_root);
        assert_eq!(row.child_modules(), *child_modules);
        assert_eq!(row.exported_surface(), *exported_surface);
    }

    let handoff_row = rows
        .iter()
        .find(|row| row.domain() == WorthQueryIntentAdmissionTopologyDomain::Handoffs)
        .expect("handoff topology row should exist");
    assert!(handoff_row
        .exported_surface()
        .contains(&"WorthQueryAuthoritativeIntentExecutionBinding"));
    assert!(handoff_row
        .ownership_detail()
        .contains("execution handoffs execution bindings"));

    let certification_row = rows
        .iter()
        .find(|row| row.domain() == WorthQueryIntentAdmissionTopologyDomain::Certification)
        .expect("certification topology row should exist");
    assert!(certification_row
        .exported_surface()
        .contains(&"certify_intent_admission"));
}
