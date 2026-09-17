mod support;

use bank_domain::model::BankPrincipalId;
use bank_server::{
    BankIdentityRuntime, BankIdentityRuntimeBuildError, BankPrincipalAdmissionError,
    BankPrincipalSeed,
};
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;
use worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_host::facade::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenialKind, WorthQueryPrincipalResolutionDenialKind,
};

use support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn unknown_dynamic_identity_is_denied() {
    let installed = DynamicIdentity::new("installed");
    let unknown = DynamicIdentity::new("unknown");
    assert_ne!(installed.external().subject(), unknown.external().subject());
    let world = runtime([BankPrincipalSeed::enabled(
        BankPrincipalId::new(51).expect("nonzero principal id should admit"),
        installed.external(),
    )]);

    assert_resolution_denial(
        block_on(world.runtime.authenticate_with(
            &world.authentication,
            CausalCredential::for_identity(&unknown),
            &request_scope(),
        )),
        WorthQueryPrincipalResolutionDenialKind::UnknownPrincipal,
    );
}

#[test]
fn disabled_dynamic_identity_is_denied() {
    let disabled = DynamicIdentity::new("disabled");
    let world = runtime([BankPrincipalSeed::new(
        BankPrincipalId::new(61).expect("nonzero principal id should admit"),
        disabled.external(),
        WorthQueryPrincipalMappingStatus::Disabled,
    )]);

    assert_resolution_denial(
        block_on(world.runtime.authenticate_with(
            &world.authentication,
            CausalCredential::for_identity(&disabled),
            &request_scope(),
        )),
        WorthQueryPrincipalResolutionDenialKind::DisabledPrincipal,
    );
}

#[test]
fn ambiguous_dynamic_identity_is_denied() {
    let ambiguous = DynamicIdentity::new("ambiguous");
    let result = BankIdentityRuntime::install([
        BankPrincipalSeed::enabled(
            BankPrincipalId::new(71).expect("nonzero principal id should admit"),
            ambiguous.external(),
        ),
        BankPrincipalSeed::enabled(
            BankPrincipalId::new(72).expect("nonzero principal id should admit"),
            ambiguous.external(),
        ),
    ]);
    let error = match result {
        Ok(_) => panic!("duplicate external identity must not publish an ambiguous graph"),
        Err(error) => error,
    };

    match error {
        BankIdentityRuntimeBuildError::ApplicationInstallation(
            WorthQueryInMemoryApplicationDenial::InitialState(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::DuplicateExternalIdentity
        ),
        other => panic!("unexpected duplicate-identity denial: {other:?}"),
    }
}

fn assert_resolution_denial<T>(
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
