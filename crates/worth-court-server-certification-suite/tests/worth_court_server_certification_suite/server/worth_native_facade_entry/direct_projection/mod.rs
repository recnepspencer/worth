#[allow(unused_imports)] use crate::worth_native_facade_entry::ordinary_context::direct_projection::*;
use {worth_proof::TransitionOutcome};
use {worth_query::facade::foundation::ProjectionAuthorityOutcome, worth_query::facade::foundation::ProjectionConsumptionBindingContext, worth_query::facade::foundation::ProjectionFactConsumptionPathError, worth_query::facade::foundation::WorthQueryConsumedProjectionAuthority};
use {worth_server::WorthServerDirectProjectionOutcome, worth_server::WorthServerDirectProjectionRequest, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerSuccessKind};

use {worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider, worth_testkit_server::shared_support_worth_native_assertions::admitted_named_read, worth_testkit_server::shared_support_worth_native_assertions::worth_native_session, worth_testkit_server::shared_support_worth_native_runtime::build_server, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider};

#[test]
fn direct_projection_consumes_retained_live_read_facts_with_typed_receipt_boundary() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let direct_read = direct_read_success(session.direct().read(&declaration));
    let request = projection_request()
        .entity_identities()
        .view_local_identities()
        .display_field("profile.display_name");
    let query_projection_outcome = direct_read
        .read_result()
        .consume_projection_authority_with_binding(
            query_binding(&request, direct_read.read_result().receipt()),
            request.authority_contract().clone(),
        )
        .expect("query projection path should stay typed");
    let (query_projection, _) = query_projection_outcome
        .into_admitted()
        .expect("query projection should admit");

    let projection = direct_projection_success(session.direct().project(&declaration, &request));

    assert_eq!(
        projection
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        worth_server::WorthServerSuccessKind::DirectProjection
    );
    assert_eq!(projection.facts().entity_identities().len(), 1);
    assert_eq!(projection.facts().view_local_identities().len(), 1);
    assert_eq!(projection.facts().display_fields().len(), 1);
    assert!(projection.warning_kinds().is_empty());
    assert_eq!(
        projection.materialization_digest().as_str(),
        query_projection
            .receipt()
            .materialized_fact_posture()
            .map(|posture| posture.posture_digest())
            .unwrap_or(query_projection.receipt().receipt_digest())
    );
    assert_eq!(
        projection.basis_digest(),
        Some(
            direct_read
                .read_result()
                .receipt()
                .snapshot_identity()
                .terminal_projection_for_reporting()
                .as_str()
        )
    );
    assert_eq!(projection.policy_digest(), "policy:test");
    assert_eq!(
        projection.result_shape_digest(),
        direct_read.read_result().receipt().view_shape_digest()
    );
    assert_eq!(
        projection
            .projection_consumption_envelope()
            .integrity_digest(),
        projection.fact_receipt().integrity_digest()
    );
    assert_eq!(
        projection.fact_receipt().counter_snapshot_digest(),
        query_projection.receipt().counter_snapshot_digest()
    );
}

#[test]
fn direct_projection_denies_hidden_field_requests() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let request = worth_server::WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id"],
    )
    .display_field("profile.display_name");

    let denial = direct_projection_denied(session.direct().project(&declaration, &request));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDenied
    );
}

#[test]
fn direct_projection_denies_fact_families_live_reads_do_not_prove() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let request = projection_request().target_identity();

    let denial = direct_projection_denied(session.direct().project(&declaration, &request));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionSourceMismatch
    );
}

#[test]
fn direct_projection_preserves_query_projection_receipt_parity() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let direct_read = direct_read_success(session.direct().read(&declaration));
    let request = projection_request()
        .entity_identities()
        .display_field("profile.display_name");

    let direct = direct_projection_success(session.direct().project(&declaration, &request));
    let query = query_projection_success(
        direct_read
            .read_result()
            .consume_projection_authority_with_binding(
                query_binding(&request, direct_read.read_result().receipt()),
                request.authority_contract().clone(),
            ),
    );

    assert_eq!(
        direct.fact_receipt().fact_set_digest(),
        query.receipt().fact_set_digest()
    );
    assert_eq!(
        direct.fact_receipt().receipt_digest(),
        query.receipt().receipt_digest()
    );
    assert_eq!(direct.basis_digest(), query.contract().basis_digest());
    assert_eq!(direct.policy_digest(), query.contract().policy_digest());
    assert_eq!(
        direct.result_shape_digest(),
        query.contract().canonical_result_shape_digest()
    );
    assert_eq!(
        direct.materialization_digest().as_str(),
        query
            .receipt()
            .materialized_fact_posture()
            .map(|posture| posture.posture_digest())
            .unwrap_or(query.receipt().receipt_digest())
    );
    assert_eq!(
        direct.fact_receipt().counter_snapshot_digest(),
        query.receipt().counter_snapshot_digest()
    );
    assert_eq!(query.counters().authority_constructions(), 1);
}

#[test]
fn direct_projection_preserves_materialized_remask_posture_in_direct_context() {
    let server = build_server_with_workspace_provider(
        worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider::new(crate::worth_native_facade_entry::ordinary_context::direct_projection::ProjectionRemaskTestSupport::projection()),
        true,
    );
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let request = projection_request()
        .entity_identities()
        .display_field("profile.display_name");

    let projection = direct_projection_success(session.direct().project(&declaration, &request));

    assert_eq!(
        projection.direct_context().remask_posture().disposition(),
        worth_server::WorthServerDirectRemaskDisposition::Remasked
    );
    assert_eq!(
        projection
            .direct_context()
            .remask_posture()
            .materialized_artifact()
            .expect("materialized remask artifact")
            .basis_digest(),
        projection
            .direct_context()
            .basis_digest()
            .expect("projection basis digest")
    );
    assert_eq!(
        projection.direct_context().remask_posture().remask_digest(),
        projection
            .facts()
            .materialized_fact_posture()
            .map(|posture| posture.posture_digest())
    );
}

#[test]
fn direct_projection_denies_requests_with_no_fact_families() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let request = worth_server::WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id", "profile.display_name"],
    );

    let denial = direct_projection_denied(session.direct().project(&declaration, &request));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::DirectProjectionBindingInvalid
    );
}

#[test]
fn direct_projection_denies_missing_retained_live_view() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile.missing");
    let request = projection_request().entity_identities();

    let denial = direct_projection_denied(session.direct().project(&declaration, &request));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::RetainedQueryArtifactUnavailable
    );
    assert_eq!(
        denial.detail(),
        "live view `users.profile.missing` has no retained subscription installation"
    );
}

#[test]
fn direct_projection_preserves_query_projection_denial_parity_for_hidden_fields() {
    let server = build_server(true);
    let session = worth_native_session(&server);
    let declaration = admitted_named_read(&session, "users.profile");
    let direct_read = direct_read_success(session.direct().read(&declaration));
    let request = worth_server::WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id"],
    )
    .display_field("profile.display_name");

    let direct_denial = direct_projection_denied(session.direct().project(&declaration, &request));
    let query_attempt = direct_read
        .read_result()
        .consume_projection_authority_with_binding(
            query_binding(&request, direct_read.read_result().receipt()),
            request.authority_contract().clone(),
        )
        .expect("query projection denial should stay typed");

    assert_eq!(
        direct_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDenied
    );
    match query_attempt {
        worth_query::facade::foundation::ProjectionAuthorityOutcome::ConsumptionDenied(denied) => {
            assert_eq!(direct_denial.detail(), format!("{:?}", denied.reason()));
        }
        other => panic!("expected denied query projection path, got {other:?}"),
    }
}
