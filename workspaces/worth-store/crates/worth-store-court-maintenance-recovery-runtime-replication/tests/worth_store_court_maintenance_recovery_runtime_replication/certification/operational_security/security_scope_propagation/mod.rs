#[allow(unused_imports)]
use crate::security_scope_propagation::*;

#[test]
fn stable_read_scope_survives_protection_observation_and_decode_entry() {
    with_stable_read_guard_scope("stable-read-preserves-scope", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let metadata = platform_page_metadata("stable-read-preserves-scope");
        let input = stable_read_scope_input(&handle, guard_scope, metadata, metadata);

        let propagation = match worth_store::physical_runtime::stability::StableReadSecurityScopePropagation::protect(input) {
            worth_proof::TransitionOutcome::Success(propagation) => propagation,
            other => panic!("stable read scope should propagate: {other:?}"),
        };
        let observed = match propagation.observe_after_root_check(handle.plan().root()) {
            worth_proof::TransitionOutcome::Success(observed) => observed,
            other => panic!("root observation should preserve propagated scope: {other:?}"),
        };
        let decode_entry = observed.logical_decode_entry_scope();

        assert_eq!(decode_entry.metadata(), metadata);
        assert_eq!(
            decode_entry.carrier_basis().page_header_generation(),
            guard_scope.reference().generation()
        );
        assert_eq!(
            decode_entry
                .carrier_basis()
                .manifest_page_slot()
                .generation(),
            guard_scope.reference().generation()
        );
        assert_eq!(decode_entry.carrier_basis().guard_scope(), guard_scope);
        assert_eq!(decode_entry.counters().store_counters().preserved(), 1);
        assert_eq!(decode_entry.counters().root_observations(), 1);
        assert_eq!(decode_entry.counters().logical_decode_entries(), 1);
    });
}

#[test]
fn stale_propagated_scope_is_physical_security_denial_before_logical_decode() {
    with_stable_read_guard_scope("stable-read-stale-scope", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let expected = platform_page_metadata("stale-scope-expected");
        let stale = platform_page_metadata_with_key_version(
            "stale-scope-observed",
            worth_store_security::StoreKeyVersionPosture::Stale,
        );
        let input = stable_read_scope_input(&handle, guard_scope, stale, expected);

        let outcome =
            worth_store::physical_runtime::stability::StableReadSecurityScopePropagation::protect(
                input,
            );

        match outcome {
            worth_proof::TransitionOutcome::Denied(denial) => {
                assert_eq!(
                    denial.store_denial().kind(),
                    worth_store_security::StoreSecurityScopePropagationDenialKind::StalePropagatedSecurityScope
                );
                assert_eq!(denial.store_denial().counters().stale(), 1);
            }
            other => panic!("stale scope must deny before logical decode entry exists: {other:?}"),
        }
    });
}

#[test]
fn security_scope_drift_between_page_header_and_manifest_denies_before_logical_decode() {
    with_stable_read_guard_scope("stable-read-scope-drift", |guard_scope| {
        let handle = stable_read_handle(guard_scope);
        let page_metadata = platform_page_metadata_with_tenant(
            "page-tenant-scope",
            worth_store_security::StoreTenantScope::TenantPhysicalBoundary,
        );
        let manifest_metadata = platform_page_metadata_with_tenant(
            "manifest-tenant-scope",
            worth_store_security::StoreTenantScope::MultiTenantPhysicalBoundary,
        );
        let input = stable_read_scope_input(&handle, guard_scope, page_metadata, manifest_metadata);

        let outcome =
            worth_store::physical_runtime::stability::StableReadSecurityScopePropagation::protect(
                input,
            );

        match outcome {
            worth_proof::TransitionOutcome::Denied(denial) => {
                assert_eq!(
                    denial.store_denial().kind(),
                    worth_store_security::StoreSecurityScopePropagationDenialKind::ScopeDriftBeforeLogicalDecode
                );
                assert_eq!(denial.store_denial().counters().drifted(), 1);
            }
            other => panic!("scope drift must deny before logical decode entry exists: {other:?}"),
        }
    });
}
