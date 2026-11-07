use super::{
    WorthQueryLiveArtifactTarget, WorthQueryLiveView, WorthQueryRuntime, WorthQueryRuntimeError,
    WorthQueryWorkspace,
};
use crate::subscription::{
    commit_prepared_subscription_lifecycle_close, prepare_subscription_lifecycle_close,
    SubscriptionLifecycleCloseRequest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveViewCloseReceipt {
    lane_terminal: bool,
    live_async_attempt_count: usize,
    live_resource_count: usize,
    live_consumer_lease_count: usize,
}

impl WorthQueryWorkspace {
    pub fn close_bridge_async_live_view<T>(
        &mut self,
        view: WorthQueryLiveView<T>,
    ) -> Result<WorthQueryLiveViewCloseReceipt, WorthQueryRuntimeError> {
        self.runtime.close_bridge_async_live_view(&view)
    }

    pub fn close_owned_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<WorthQueryLiveViewCloseReceipt, WorthQueryRuntimeError> {
        self.runtime.close_bridge_async_live_view(view)
    }
}

impl WorthQueryRuntime {
    fn close_bridge_async_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<WorthQueryLiveViewCloseReceipt, WorthQueryRuntimeError> {
        let target = WorthQueryLiveArtifactTarget::from_subscription_installation(
            view.subscription_installation(),
        );
        let state = self
            .live_subscriptions
            .get(&target)
            .ok_or_else(|| close_error(view.name(), "the Query live resource is not registered"))?;
        if state.installation != *view.subscription_installation() {
            return Err(close_error(
                view.name(),
                "the live-view installation does not match Query's registered resource",
            ));
        }
        if state.async_source_binding.is_none() {
            return Err(close_error(
                view.name(),
                "the live resource is not bound to a bridge async attempt",
            ));
        }
        if self.derived_dependency_index.has_live_dependents(&target)
            || self.effect_index.has_live_dependents(&target)
        {
            return Err(close_error(
                view.name(),
                "the live resource still has computed or effect dependents",
            ));
        }

        let close_request =
            SubscriptionLifecycleCloseRequest::TerminateConsumer(state.consumer_attachment.clone());
        let prepared = prepare_subscription_lifecycle_close(
            &self.active_subscriptions,
            &state.active_lane_handle,
            close_request,
        )
        .map_err(|error| {
            close_error(
                view.name(),
                format!("{}: {}", error.denial_kind().as_str(), error.message()),
            )
        })?;
        self.backend
            .close_live_view(view.name())
            .map_err(|error| close_error(view.name(), error.to_string()))?;
        let closeout =
            commit_prepared_subscription_lifecycle_close(&mut self.active_subscriptions, prepared);

        self.live_subscriptions.remove(&target);
        self.materialized_read_views.remove(&target);
        self.live_subscription_index.unregister(&target);
        self.unregister_installed_live_route(&target);

        Ok(WorthQueryLiveViewCloseReceipt {
            lane_terminal: closeout.lane_terminal(),
            live_async_attempt_count: self
                .live_subscriptions
                .values()
                .filter(|state| state.async_source_binding.is_some())
                .count(),
            live_resource_count: self.live_subscriptions.len(),
            live_consumer_lease_count: self.active_subscriptions.attachment_count(),
        })
    }
}

impl WorthQueryLiveViewCloseReceipt {
    pub fn lane_terminal(self) -> bool {
        self.lane_terminal
    }

    pub fn live_async_attempt_count(self) -> usize {
        self.live_async_attempt_count
    }

    pub fn live_resource_count(self) -> usize {
        self.live_resource_count
    }

    pub fn live_consumer_lease_count(self) -> usize {
        self.live_consumer_lease_count
    }
}

fn close_error(view_name: &str, message: impl Into<String>) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage: "bridge-async-live-view-close",
        message: message.into(),
    }
}
