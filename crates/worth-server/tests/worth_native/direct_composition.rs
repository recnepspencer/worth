use std::sync::atomic::Ordering;
use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerDirectDeclaration, WorthServerDirectProjectionOutcome,
    WorthServerDirectProjectionRequest, WorthServerDirectReadOutcome,
    WorthServerDirectStateOutcome, WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation,
    WorthServerSuccessKind, WorthServerSurfaceFamily, WorthServerTransportClass,
};

use {
    crate::shared_support_direct_context_runtime::RemaskWorkspaceProvider,
    crate::shared_support_query_handoff_fixture::{
        admit_read_posture, request_input, resolve_request_context, success,
    },
    crate::shared_support_worth_native_assertions::{
        admitted_named_read, direct_provenance_digest, family_contract_digest,
        response_provenance_digest, worth_native_session,
    },
    crate::shared_support_worth_native_runtime::{
        build_server, build_server_with_profiled_counting_workspace,
        build_server_with_workspace_provider,
    },
};

#[test]
fn direct_product_flow_matches_lower_direct_path_without_endpoint_glue() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let product = session
        .direct()
        .product()
        .named_read("users.profile")
        .expect("product flow should prepare and admit");
    let declaration = admitted_named_read(&session, "users.profile");
    let request = projection_request()
        .entity_identities()
        .view_local_identities()
        .display_field("profile.display_name");

    let product_read = direct_read_success(product.read());
    let direct_read = direct_read_success(session.direct().read(&declaration));
    let product_projection = direct_projection_success(product.project(&request));
    let direct_projection =
        direct_projection_success(session.direct().project(&declaration, &request));

    assert_eq!(
        product.declaration_snapshot().declaration_digest(),
        declaration.declaration_digest()
    );
    assert_eq!(
        product.declaration_snapshot().family_contract_digest(),
        declaration.query_family_contract().contract_digest()
    );
    assert_eq!(
        product_read.canonical_digest(),
        direct_read.canonical_digest()
    );
    assert_eq!(
        product_projection.canonical_digest(),
        direct_projection.canonical_digest()
    );
    assert_eq!(
        product_projection.fact_receipt().receipt_digest(),
        direct_projection.fact_receipt().receipt_digest()
    );
}

#[test]
fn direct_product_flow_preserves_compatibility_overlap_without_semantic_drift() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let product = session
        .direct()
        .product()
        .named_read("users.profile")
        .expect("product flow should prepare and admit");

    let product_read = direct_read_success(product.read());
    let compatibility = success(
        server
            .query_handoff()
            .prepare(WorthServerQueryHandoffInput::new(
                admit_read_posture(
                    &server,
                    resolve_request_context(
                        &server,
                        request_input(
                            WorthServerSurfaceFamily::CompatHttp,
                            WorthServerTransportClass::CompatHttp,
                        ),
                    ),
                ),
                WorthServerQueryHandoffOperation::query_read("users.profile"),
            )),
    );

    assert_eq!(
        product.declaration_snapshot().family_contract_digest(),
        family_contract_digest(compatibility.support_posture())
    );
    assert_eq!(
        product_read.workspace_name(),
        compatibility.workspace().name()
    );
    assert_eq!(
        product_read.direct_context().branch_digest(),
        compatibility
            .admission()
            .request_context()
            .branch_target()
            .branch_digest()
    );
    assert_eq!(
        product_read.direct_context().workspace_digest(),
        compatibility
            .admission()
            .request_context()
            .workspace_target()
            .workspace_digest()
    );
}

#[test]
fn direct_product_flow_preserves_cost_honesty_at_expensive_seams() {
    let (server, attempted_writes) = build_server_with_profiled_counting_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
    );
    let session = worth_native_session(&server);
    let product = session
        .direct()
        .product()
        .named_read("users.profile")
        .expect("product flow should prepare and admit");
    let declaration = admitted_named_read(&session, "users.profile");
    let request = projection_request()
        .entity_identities()
        .display_field("profile.display_name");

    let product_retained = direct_retained_posture_success(product.product_retained_posture());
    let direct_state = direct_state_success(session.direct().state(&declaration));
    let product_projection = direct_projection_success(product.project(&request));
    let direct_projection =
        direct_projection_success(session.direct().project(&declaration, &request));

    assert_eq!(
        family_contract_digest(product_retained.support_posture()),
        family_contract_digest(direct_state.support_posture())
    );
    assert_eq!(
        product_retained.basis_digest(),
        direct_state.direct_context().basis_digest()
    );
    assert_eq!(
        product_retained.runtime_state().state_digest(),
        direct_state.runtime_state().state_digest()
    );
    assert_eq!(
        product_projection.fact_receipt().counter_snapshot_digest(),
        direct_projection.fact_receipt().counter_snapshot_digest()
    );
    assert_eq!(
        product_projection.materialization_digest().as_str(),
        direct_projection.materialization_digest().as_str()
    );
    assert_eq!(
        product_projection
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DirectProjection
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "product composition must not conceal write-path work on read/state/projection lanes"
    );
}

#[test]
fn direct_product_flow_leaves_no_endpoint_style_residue_for_snapshot_provenance_or_denials() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let product = session
        .direct()
        .product()
        .named_read("users.profile")
        .expect("product flow should prepare and admit");
    let missing = session
        .direct()
        .product()
        .named_read("users.profile.missing")
        .expect("missing live view still admits declaration shape");
    let read = direct_read_success(product.read());
    let denial = direct_read_denied(missing.read());

    assert_eq!(
        product.declaration_snapshot().family_contract_digest(),
        family_contract_digest(read.support_posture())
    );
    assert_eq!(
        direct_provenance_digest(read.direct_context().provenance()),
        response_provenance_digest(read.response_envelope())
    );
    assert_eq!(
        denial.detail(),
        "live view `users.profile.missing` has no retained subscription installation"
    );
}

#[test]
fn direct_product_root_preserves_declaration_intake_denials_without_route_only_glue() {
    let server = build_server(true);
    let session = worth_native_session(&server);

    let denial = session
        .direct()
        .product()
        .read(WorthServerDirectDeclaration::saved_query(
            "users.profile.saved",
        ))
        .expect_err("saved-query declaration intake should remain denied at the product root");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerDirectDeclarationDenialCode::SourceNotAdmitted
    );
    assert_eq!(
        denial
            .support_snapshot()
            .expect("product-root declaration denial should preserve support snapshot")
            .source_support_reason(),
        "saved-query declaration intake remains deferred until a later direct-consumption phase"
    );
}

#[test]
fn direct_product_flow_carries_retained_posture_and_typed_fact_receipts_without_extra_status_glue()
{
    let server = build_server_with_workspace_provider(
        RemaskWorkspaceProvider::new(ProjectionRemaskTestSupport::projection()),
        true,
    );
    let session = worth_native_session(&server);
    let product = session
        .direct()
        .product()
        .named_read("users.profile")
        .expect("product flow should prepare and admit");
    let request = projection_request()
        .entity_identities()
        .display_field("profile.display_name");

    let retained = direct_retained_posture_success(product.product_retained_posture());
    let state = direct_state_success(product.state());
    let projection = direct_projection_success(product.project(&request));

    assert_eq!(
        retained.declaration_snapshot().declaration_digest(),
        product.declaration_snapshot().declaration_digest()
    );
    assert_eq!(
        retained.runtime_state().state_digest(),
        state.runtime_state().state_digest()
    );
    assert_eq!(
        retained.basis_digest(),
        state.direct_context().basis_digest()
    );
    assert_eq!(
        retained.remask_posture().remask_digest(),
        state.direct_context().remask_posture().remask_digest()
    );
    assert_eq!(
        retained
            .async_result_state()
            .map(|state| state.inner().result_state_for_reporting()),
        state
            .async_result_state()
            .as_ref()
            .map(|state| state.inner().result_state_for_reporting())
    );
    assert_eq!(
        retained
            .temporal_state()
            .map(|state| state.inner().state_digest()),
        state
            .temporal_state()
            .as_ref()
            .map(|state| state.inner().state_digest())
    );
    assert_eq!(
        projection.fact_receipt().materialization_digest().as_str(),
        projection.materialization_digest().as_str()
    );
}

fn projection_request() -> WorthServerDirectProjectionRequest {
    WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id", "profile.display_name"],
    )
}

fn direct_read_success(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
    }
}

fn direct_read_denied(
    outcome: WorthServerDirectReadOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct read, got {other:?}"),
    }
}

fn direct_state_success(
    outcome: WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

fn direct_retained_posture_success(
    outcome: worth_proof::TransitionOutcome<
        worth_server::WorthServerDirectRetainedPosture,
        worth_server::WorthServerQueryHandoffDenial,
        worth_server::WorthServerQueryHandoffDeferred,
        worth_server::WorthServerQueryHandoffStale,
        worth_server::WorthServerQueryHandoffRebindRequired,
        worth_server::WorthServerQueryHandoffFailure,
    >,
) -> worth_server::WorthServerDirectRetainedPosture {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful retained posture, got {other:?}"),
    }
}

fn direct_projection_success(
    outcome: WorthServerDirectProjectionOutcome,
) -> worth_server::WorthServerDirectProjection {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct projection, got {other:?}"),
    }
}

struct ProjectionRemaskTestSupport;

impl ProjectionRemaskTestSupport {
    fn projection() -> worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection {
        worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
            worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
            "policy:test",
            "tenant-truth:test",
            "tenant-schema:test",
            "relationship-proof:test",
            "schema-context:test",
        )
    }
}
