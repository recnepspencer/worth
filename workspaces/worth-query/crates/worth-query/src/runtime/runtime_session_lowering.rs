use super::live_subscription::{
    live_subscription_source_identity, live_subscription_view_shape_source_identity,
};
use super::*;
use crate::subscription::SubscriptionActivationInput;

pub(super) struct LoweredRuntimeLiveSubscriptionRequest {
    pub(super) query_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) canonical_query_digest: crate::identity::CanonicalQueryDigest,
    pub(super) live_view_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) canonical_result_shape_digest: crate::identity::CanonicalResultShapeDigest,
    pub(super) subscription_family: crate::subscription::QuerySubscriptionFamily,
    pub(super) subscription_declaration_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) admission_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) bridge_declaration_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) basis_binding_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) signal_strategy_identity: crate::WorthQueryEvidenceIdentity,
    pub(super) activation: SubscriptionActivationInput,
}

pub(super) fn lower_runtime_live_subscription_request(
    backend: &dyn WorthQueryRuntimeBackend,
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
    schema_view: QuerySchemaView,
    future_selection: crate::subscription::QuerySubscriptionFutureSelection,
) -> Result<LoweredRuntimeLiveSubscriptionRequest, WorthQueryRuntimeError> {
    let session = declare_runtime_live_query_session_with_grouped_baseline(
        request.clone(),
        schema_view,
        backend
            .current_snapshot_identity()?
            .admit_runtime_backend_authority(),
        grouped_baseline_members_or_error(backend, view_name, request)?,
    )
    .map_err(|error| live_subscription_error(view_name, "live-lowering", error))?;
    lower_runtime_live_subscription_session(view_name, request, session, future_selection)
}

pub(super) fn lower_runtime_live_subscription_read_binding(
    backend: &dyn WorthQueryRuntimeBackend,
    view_name: &str,
    binding: &WorthQueryReadExecutionBinding,
) -> Result<LoweredRuntimeLiveSubscriptionRequest, WorthQueryRuntimeError> {
    let read_graph = binding.read_family().read_graph();
    let request = read_graph.declarative_request();
    let session = crate::declarative_live::declare_runtime_live_query_session_from_admitted_read(
        request.clone(),
        read_graph.canonical().clone(),
        read_graph.validated().clone(),
        read_graph.execution_plan().clone(),
        backend
            .current_snapshot_identity()?
            .admit_runtime_backend_authority(),
        grouped_baseline_members_or_error(backend, view_name, request)?,
    )
    .map_err(|error| live_subscription_error(view_name, "admitted-read-live-lowering", error))?;
    lower_runtime_live_subscription_session(
        view_name,
        request,
        session,
        crate::subscription::QuerySubscriptionFutureSelection::ordinary(),
    )
}

fn lower_runtime_live_subscription_session(
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
    session: crate::declarative_live::DeclarativeLiveQuerySession,
    future_selection: crate::subscription::QuerySubscriptionFutureSelection,
) -> Result<LoweredRuntimeLiveSubscriptionRequest, WorthQueryRuntimeError> {
    let view_family = session.live_view().lowering().family();
    let dimensions = subscription_dimensions_for_request(request, view_family)?;
    let scoped_declaration_basis = crate::basis_lifecycle::basis_lifecycle()
        .current_head()
        .declare_subscription()
        .map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "basis-declaration",
                message: format!("{error:?}"),
            },
        )?;
    let live_admission =
        crate::subscription::LiveQueryAdmissionArtifact::from_live_promotion_with_view_and_future_selection(
            session.live_view().core_live_plan().descriptor(),
            scoped_declaration_basis,
            view_family,
            future_selection,
            dimensions,
        );
    let selection = select_runtime_subscription_family(view_name, live_admission)?;
    let subscription_family = selection.family().clone();
    let declaration =
        declare_query_subscription(selection, runtime_slice_budget()).map_err(|error| {
            WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "declaration",
                message: format!("{error:?}"),
            }
        })?;
    let lowering =
        lower_query_subscription_to_bridge(declaration, runtime_bridge_lowering_budget()).map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "bridge-lowering",
                message: format!("{error:?}"),
            },
        )?;
    let admission = admit_query_subscription(lowering, runtime_subscription_admission_budget())
        .map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "subscription-admission",
                message: format!("{error:?}"),
            },
        )?;

    Ok(LoweredRuntimeLiveSubscriptionRequest {
        query_identity: live_subscription_source_identity(
            "query_execution_plan",
            &session
                .view_plan()
                .execution_plan()
                .query()
                .plan_digest()
                .evidence_identity(),
        ),
        canonical_query_digest: session.canonical().query().digest().clone(),
        live_view_identity: live_subscription_source_identity(
            "live_view",
            &live_subscription_view_shape_source_identity(view_family),
        ),
        canonical_result_shape_digest: session.canonical().result_shape().digest().clone(),
        subscription_family,
        subscription_declaration_identity: live_subscription_source_identity(
            "subscription_declaration",
            admission.query_declaration_identity(),
        ),
        admission_identity: live_subscription_source_identity(
            "admission",
            admission.evidence_identity(),
        ),
        bridge_declaration_identity: live_subscription_source_identity(
            "bridge_declaration",
            admission.bridge_declaration_identity(),
        ),
        basis_binding_identity: live_subscription_source_identity(
            "basis_binding",
            admission.basis_binding_identity(),
        ),
        signal_strategy_identity: live_subscription_source_identity(
            "signal_strategy",
            admission.signal_strategy_identity(),
        ),
        activation: prepare_subscription_activation(admission),
    })
}

pub(super) fn grouped_baseline_members_or_error(
    backend: &dyn WorthQueryRuntimeBackend,
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
) -> Result<
    Option<Vec<crate::view_shape_live::WorthQueryGroupedBaselineMember>>,
    WorthQueryRuntimeError,
> {
    backend.grouped_baseline_members(request).map_err(|error| {
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "grouped-baseline",
            message: error.to_string(),
        }
    })
}

pub(super) fn select_runtime_subscription_family(
    view_name: &str,
    live_admission: crate::subscription::LiveQueryAdmissionArtifact,
) -> Result<crate::subscription::QuerySubscriptionFamilySelection, WorthQueryRuntimeError> {
    select_query_subscription_family(live_admission, runtime_family_budget()).map_err(|error| {
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "family-selection",
            message: format!("{error:?}"),
        }
    })
}

pub(super) fn install_live_subscription_activation(
    backend: &mut dyn WorthQueryRuntimeBackend,
    view_name: &str,
    activation: &SubscriptionActivationInput,
) -> Result<SubscriptionActivationReceipt, WorthQueryRuntimeError> {
    let activation_receipt = backend
        .install_live_subscription(view_name, activation)
        .map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "activation-admission",
                message: error.to_string(),
            },
        )?;
    if let Some(message) = activation_receipt.drift_from_activation(view_name, activation) {
        return Err(WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "activation-receipt",
            message,
        });
    }
    Ok(activation_receipt)
}
