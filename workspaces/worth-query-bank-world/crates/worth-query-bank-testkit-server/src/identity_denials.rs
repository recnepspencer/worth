pub use crate::facade::support;
pub use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

pub use bank_domain::model::BankPrincipalId;
pub use bank_server::{
    BankIdentityRuntime, BankIdentityRuntimeBuildError, BankPrincipalAdmissionError,
    BankPrincipalSeed,
};
pub use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
pub use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_query_host::facade::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryPrincipalResolutionDenialKind,
};

pub use support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

pub fn assert_resolution_denial<T>(
    result: Result<T, BankPrincipalAdmissionError>,
    expected: WorthQueryPrincipalResolutionDenialKind,
) {
    match result {
        Err(BankPrincipalAdmissionError::Resolution(denial)) => {
            assert_eq!(denial.kind(), expected);
        }
        Err(other) => panic!("unexpected bank principal denial: {other:?}"),
        Ok(_) => panic!("hostile identity must not resolve"),
    }
}
