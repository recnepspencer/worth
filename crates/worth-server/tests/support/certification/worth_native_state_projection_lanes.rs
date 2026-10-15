use worth_server::WorthServer;

use crate::shared_support_worth_native_assertions::admitted_named_read;

use crate::certification_suite::certification_bundle::{
    WorthServerCertificationBundle, WorthServerCertificationOutputDigest as Output,
};

use super::worth_native_common::{
    direct_bundle, direct_projection_success, direct_retained_posture_success,
    direct_state_success, projection_request, support_posture_digest,
    worth_native_session_for_branch,
};

pub fn product_retained_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let product = session
        .direct()
        .product()
        .named_read(operation_name)
        .expect("retained posture should admit");
    let retained = direct_retained_posture_success(product.product_retained_posture());
    let state = direct_state_success(product.state());
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        state.response_envelope().clone(),
    )
    .with_output_digest(
        Output::SurfaceContract,
        product.declaration_snapshot().family_contract_digest(),
    )
    .with_output_digest(
        Output::Declaration,
        product.declaration_snapshot().declaration_digest(),
    )
    .with_output_digest(Output::Handoff, state.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(retained.support_posture()),
    )
    .with_output_digest(Output::Branch, retained.direct_context().branch_digest())
    .with_output_digest(
        Output::Workspace,
        retained.direct_context().workspace_digest(),
    )
    .with_output_digest(
        Output::RetainedState,
        retained
            .runtime_state()
            .state_digest()
            .terminal_projection_for_reporting(),
    )
    .with_optional_output_digest(Output::Basis, retained.basis_digest())
    .with_optional_output_digest(Output::Remask, retained.remask_posture().remask_digest())
    .with_optional_output_digest(
        Output::AsyncResult,
        retained
            .async_result_state()
            .map(|state| state.inner().result_state_for_reporting()),
    )
    .with_optional_output_digest(
        Output::TemporalState,
        retained.temporal_state().map(|state| {
            state
                .inner()
                .state_digest()
                .terminal_projection_for_reporting()
        }),
    )
}

pub fn lower_direct_state_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let state = direct_state_success(session.direct().state(&declaration));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        state.response_envelope().clone(),
    )
    .with_output_digest(
        Output::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(Output::Declaration, declaration.declaration_digest())
    .with_output_digest(Output::Handoff, state.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(state.support_posture()),
    )
    .with_output_digest(Output::Branch, state.direct_context().branch_digest())
    .with_output_digest(Output::Workspace, state.direct_context().workspace_digest())
    .with_output_digest(
        Output::RetainedState,
        state
            .runtime_state()
            .state_digest()
            .terminal_projection_for_reporting(),
    )
    .with_optional_output_digest(Output::Basis, state.direct_context().basis_digest())
    .with_optional_output_digest(
        Output::Remask,
        state.direct_context().remask_posture().remask_digest(),
    )
    .with_optional_output_digest(
        Output::AsyncResult,
        state
            .async_result_state()
            .as_ref()
            .map(|value| value.inner().result_state_for_reporting()),
    )
    .with_optional_output_digest(
        Output::TemporalState,
        state.temporal_state().as_ref().map(|value| {
            value
                .inner()
                .state_digest()
                .terminal_projection_for_reporting()
        }),
    )
}

pub fn product_projection_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let product = session
        .direct()
        .product()
        .named_read(operation_name)
        .expect("projection should admit");
    let projection = direct_projection_success(product.project(&projection_request()));
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        projection.response_envelope().clone(),
    )
    .with_output_digest(
        Output::SurfaceContract,
        product.declaration_snapshot().family_contract_digest(),
    )
    .with_output_digest(
        Output::Declaration,
        product.declaration_snapshot().declaration_digest(),
    )
    .with_output_digest(Output::Handoff, projection.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(projection.support_posture()),
    )
    .with_output_digest(Output::Branch, projection.direct_context().branch_digest())
    .with_output_digest(
        Output::Workspace,
        projection.direct_context().workspace_digest(),
    )
    .with_optional_output_digest(Output::Basis, projection.basis_digest())
    .with_optional_output_digest(
        Output::Remask,
        projection.direct_context().remask_posture().remask_digest(),
    )
    .with_output_digest(Output::Policy, projection.policy_digest())
    .with_output_digest(
        Output::FactReceipt,
        projection.fact_receipt().receipt_digest(),
    )
    .with_output_digest(
        Output::Materialization,
        projection.materialization_digest().as_str(),
    )
    .with_output_digest(
        Output::CounterSnapshot,
        projection.fact_receipt().counter_snapshot_digest(),
    )
}

pub fn lower_direct_projection_bundle(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCertificationBundle {
    let session = worth_native_session_for_branch(server, None);
    let declaration = admitted_named_read(&session, operation_name);
    let projection = direct_projection_success(
        session
            .direct()
            .project(&declaration, &projection_request()),
    );
    direct_bundle(
        server,
        session.resolved_request_context().request_context(),
        projection.response_envelope().clone(),
    )
    .with_output_digest(
        Output::SurfaceContract,
        declaration.query_family_contract().contract_digest(),
    )
    .with_output_digest(Output::Declaration, declaration.declaration_digest())
    .with_output_digest(Output::Handoff, projection.handoff_digest())
    .with_output_digest(
        Output::SupportPosture,
        support_posture_digest(projection.support_posture()),
    )
    .with_output_digest(Output::Branch, projection.direct_context().branch_digest())
    .with_output_digest(
        Output::Workspace,
        projection.direct_context().workspace_digest(),
    )
    .with_optional_output_digest(Output::Basis, projection.basis_digest())
    .with_optional_output_digest(
        Output::Remask,
        projection.direct_context().remask_posture().remask_digest(),
    )
    .with_output_digest(Output::Policy, projection.policy_digest())
    .with_output_digest(
        Output::FactReceipt,
        projection.fact_receipt().receipt_digest(),
    )
    .with_output_digest(
        Output::Materialization,
        projection.materialization_digest().as_str(),
    )
    .with_output_digest(
        Output::CounterSnapshot,
        projection.fact_receipt().counter_snapshot_digest(),
    )
}
