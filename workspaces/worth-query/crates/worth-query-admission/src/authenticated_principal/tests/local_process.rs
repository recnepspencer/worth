//! The local principal mints through the same admitted checks as every adapter.
use super::*;
use worth_query_declaration::facade::authentication::{
    WorthQueryAuthenticationAudienceIdentity, WorthQueryAuthenticationMethodIdentity,
};

worth_query_declaration::worth_query_principal_issuer!(BoardLocal);
worth_query_declaration::worth_query_authentication_audience!(Board);
worth_query_declaration::worth_query_authentication_audience!(Other);
worth_query_declaration::worth_query_authentication_method!(LocalProcess);
worth_query_declaration::worth_query_authentication_method!(Remote);

fn local(
    audience: impl WorthQueryAuthenticationAudienceIdentity,
    method: impl WorthQueryAuthenticationMethodIdentity,
    lifetime: Duration,
) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        WorthQueryExternalPrincipalIdentity::new(BoardLocal, "owner").unwrap(),
        audience,
        method,
        lifetime,
    )
    .unwrap()
}

fn admit(
    principal: WorthQueryLocalProcessPrincipal,
    clock: WorthQueryAdmissionClock,
) -> WorthQueryAdmittedAuthenticationAdapter<TestSchema, WorthQueryLocalProcessPrincipal> {
    admit_authentication_adapter(
        &installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(Board),
            WorthQueryAuthenticationMethod::of(LocalProcess),
            clock,
        ),
        principal,
    )
    .unwrap()
}

fn denial(
    adapter: &WorthQueryAdmittedAuthenticationAdapter<TestSchema, WorthQueryLocalProcessPrincipal>,
    scope: &WorthQueryRequestScope,
) -> (
    WorthQueryAuthenticationDenialKind,
    WorthQueryAuthenticationDenialKind,
) {
    let synchronous = adapter.authenticate_local(scope).unwrap_err().kind();
    let asynchronous = block_on(adapter.authenticate((), scope))
        .unwrap_err()
        .kind();
    (synchronous, asynchronous)
}

#[test]
fn local_principal_refuses_wrong_audience_method_clock_and_interruption_like_any_adapter() {
    let system = WorthQueryAdmissionClock::system();
    let wrong_audience = admit(
        local(Other, LocalProcess, Duration::from_secs(60)),
        system.clone(),
    );
    let wrong_method = admit(
        local(Board, Remote, Duration::from_secs(60)),
        system.clone(),
    );
    let foreign = admit(
        local(Board, LocalProcess, Duration::from_secs(60)),
        WorthQueryAdmissionTimeController::new().unwrap().clock(),
    );
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), system.clone());
    let valid = admit(local(Board, LocalProcess, Duration::from_secs(60)), system);
    use WorthQueryAuthenticationDenialKind as Kind;
    for (adapter, scope, expected) in [
        (&wrong_audience, live_scope(), Kind::AudienceMismatch),
        (&wrong_method, live_scope(), Kind::MethodMismatch),
        (&foreign, live_scope(), Kind::ForeignClock),
        (&valid, cancelled, Kind::Cancelled),
    ] {
        assert_eq!(denial(adapter, &scope), (expected, expected));
    }
}

#[test]
fn local_principal_mints_one_finite_admitted_proof_on_the_installed_clock() {
    let time = WorthQueryAdmissionTimeController::new().unwrap();
    let held = time.hold();
    let adapter = admit(
        local(Board, LocalProcess, Duration::from_secs(90)),
        time.clock(),
    );
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        time.clock(),
    );
    let proof = adapter.authenticate_local(&scope).unwrap();
    assert_eq!(proof.adapter_identity(), adapter.adapter_identity());
    assert_eq!(proof.identity(), adapter.adapter().identity());
    assert_eq!(
        proof
            .expires_at()
            .duration_since(proof.validated_at())
            .unwrap(),
        Duration::from_secs(90)
    );
    assert!(proof.valid_until() <= held + Duration::from_secs(90));
    assert!(proof.valid_until() > held + Duration::from_secs(89));
    time.advance(Duration::from_secs(90)).unwrap();
    assert!(
        proof.is_expired(),
        "a local proof expires on the installed clock"
    );
}
