use worth_runtime_bridge::facade::{
    AdmittedBridgeAsyncCompletion, AdmittedBridgeAsyncRequestIdentity,
    BridgeMixedCauseOrderingInput,
};
use worth_signal::facade::NodeId;
use worth_ui_query_binding::{
    certification::{remasked_scalar_projection_workspace, scalar_projection_workspace},
    UiProjectionConsumptionBudget, UiProjectionFieldRequirement, UiScalarProjectionBatchOutcome,
    UiScalarProjectionBinding, UiScalarProjectionBindingAdmission, UiScalarProjectionFactReceipt,
    UiScalarProjectionRegistration, UiScalarProjectionTransitionReceipt, WorthUiQueryWorkspaceExt,
};

use super::async_fixture::{
    admitted_async_request_and_completion, authoritative_async_basis, projection_bridge,
    scalar_async_view,
};

pub(crate) type AsyncView = worth_query::facade::runtime::WorthQueryLiveView<
    worth_query::facade::runtime::WorthQueryUnrefinedLiveShape,
>;

pub(crate) struct ScalarLifecycleWorld {
    pub(crate) bridge: worth_runtime_bridge::facade::RuntimeBridge,
    pub(crate) workspace: worth_query::facade::runtime::WorthQueryWorkspace,
    pub(crate) view: AsyncView,
    pub(crate) binding: UiScalarProjectionBinding,
    pub(crate) request: AdmittedBridgeAsyncRequestIdentity,
    pub(crate) entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
}

impl ScalarLifecycleWorld {
    pub(crate) fn standard(node: NodeId, status: &str) -> (Self, AdmittedBridgeAsyncCompletion) {
        let bridge = projection_bridge();
        let (request, completion) = admitted_async_request_and_completion(
            &bridge,
            node,
            authoritative_async_basis("commit-initial", "snapshot-initial"),
            64,
        );
        (
            Self::with_workspace(scalar_projection_workspace(true), bridge, request, status),
            completion,
        )
    }

    pub(crate) fn retryable(node: NodeId, status: &str) -> Self {
        let bridge = projection_bridge();
        let request =
            worth_runtime_bridge::certification_for_test::retryable_async_request_for_test(
                &bridge,
                node,
                authoritative_async_basis("commit-stable", "snapshot-stable"),
            );
        Self::with_workspace(scalar_projection_workspace(true), bridge, request, status)
    }

    pub(crate) fn remasked(node: NodeId, status: &str) -> (Self, AdmittedBridgeAsyncCompletion) {
        let bridge = projection_bridge();
        let (request, completion) = admitted_async_request_and_completion(
            &bridge,
            node,
            authoritative_async_basis("commit-remasked", "snapshot-remasked"),
            64,
        );
        (
            Self::with_workspace(
                remasked_scalar_projection_workspace(),
                bridge,
                request,
                status,
            ),
            completion,
        )
    }

    fn with_workspace(
        mut workspace: worth_query::facade::runtime::WorthQueryWorkspace,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        request: AdmittedBridgeAsyncRequestIdentity,
        status: &str,
    ) -> Self {
        let entity = worth_ui_query_binding::certification::insert_projection_status(
            &mut workspace,
            "platform.pulse.status",
            status,
        );
        let view = scalar_async_view(&mut workspace, &request);
        let binding = scalar_binding(&workspace);
        Self {
            bridge,
            workspace,
            view,
            binding,
            request,
            entity,
        }
    }

    pub(crate) fn initial(&mut self) -> UiScalarProjectionTransitionReceipt {
        self.binding
            .consume_initial_async_result(
                &mut self.workspace,
                &self.view,
                UiProjectionConsumptionBudget::platform_pulse(),
            )
            .expect("initial Query Pending must advance exactly once")
    }

    pub(crate) fn advance(
        &mut self,
        input: BridgeMixedCauseOrderingInput,
        predecessor: Option<UiScalarProjectionFactReceipt>,
    ) -> UiScalarProjectionTransitionReceipt {
        let batch = self.transition_batch(input);
        match self.binding.consume_async_result_batch(
            &mut self.workspace,
            batch,
            predecessor,
            UiProjectionConsumptionBudget::platform_pulse(),
        ) {
            UiScalarProjectionBatchOutcome::Advanced(receipt) => receipt,
            UiScalarProjectionBatchOutcome::Unchanged(_) => {
                panic!("QP02 lifecycle stimulus must advance")
            }
        }
    }

    pub(crate) fn transition_batch(
        &mut self,
        input: BridgeMixedCauseOrderingInput,
    ) -> worth_query::facade::runtime::WorthQueryAsyncResultTransitionBatch {
        let request = worth_runtime_bridge::facade::BridgeMixedCauseOrderingRequest::new(
            worth_runtime_bridge::facade::BridgeMixedCauseOrderingLaneKind::Authoritative,
            vec![input],
        );
        let ordering = self.bridge.order_mixed_causes(&request);
        self.workspace
            .admit_bridge_async_result_transitions(&self.view, &ordering)
            .expect("Bridge-issued transition must reach Query")
    }
}

pub(crate) fn unsupported_admission() -> UiScalarProjectionBindingAdmission {
    let workspace = scalar_projection_workspace(false);
    let view = workspace
        .worth_ui()
        .expect("Worth UI domain installed")
        .projection_view("platform.pulse.status")
        .expect("Platform Pulse view installed");
    UiScalarProjectionRegistration::text(
        view,
        UiProjectionFieldRequirement::declared("status").expect("static field must admit"),
    )
    .admit(&workspace)
}

fn scalar_binding(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> UiScalarProjectionBinding {
    let view = workspace
        .worth_ui()
        .expect("Worth UI domain installed")
        .projection_view("platform.pulse.status")
        .expect("Platform Pulse view installed");
    let registration = UiScalarProjectionRegistration::text(
        view,
        UiProjectionFieldRequirement::declared("status").expect("static field must admit"),
    );
    match registration.admit(workspace) {
        UiScalarProjectionBindingAdmission::Ready(binding) => binding,
        UiScalarProjectionBindingAdmission::Unavailable(unavailable) => {
            panic!("supported QP02 world was unavailable: {unavailable:?}")
        }
        UiScalarProjectionBindingAdmission::Stopped(stop) => {
            panic!("QP02 scalar binding stopped: {stop:?}")
        }
    }
}
