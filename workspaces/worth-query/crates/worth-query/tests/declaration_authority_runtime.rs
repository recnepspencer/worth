use worth_query::facade::runtime::{
    WorthQueryDeclarationAuthorityRuntime, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupportStatus,
};

const DECLARATION_AUTHORITY_DENIAL: &str =
    "declaration-authority runtimes do not own query execution";

#[test]
fn declaration_authority_runtime_has_no_execution_family() {
    let runtime = WorthQueryDeclarationAuthorityRuntime::builder().build();
    let profile = runtime.support_profile();

    for family in WorthQueryRuntimeFacadeFamily::ALL {
        let support = profile
            .support_for(family)
            .expect("every public runtime family must declare its posture");
        assert_eq!(
            support.status(),
            WorthQueryRuntimeFamilySupportStatus::Unsupported
        );
        assert_eq!(support.denial_reason(), Some(DECLARATION_AUTHORITY_DENIAL));
        assert!(!support.ordinary_downstream_dx());
    }
}
