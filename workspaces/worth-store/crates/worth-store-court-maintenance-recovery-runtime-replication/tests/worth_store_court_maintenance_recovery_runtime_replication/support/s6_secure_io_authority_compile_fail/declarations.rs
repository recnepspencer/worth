pub(crate) use std::path::Path;

pub(crate) use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

pub(crate) const FIXTURES: &[(&str, &[&str])] = &[(
    "identity_provider_claim_cannot_satisfy_secure_io_scope.rs",
    &["IoSchedulerSecurityScopeAdmission", "StoreJwtSubjectClaim"],
)];

pub(crate) fn store_workspace_root() -> &'static Path {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}
