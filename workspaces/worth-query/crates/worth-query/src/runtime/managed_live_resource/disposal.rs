use super::super::*;
use crate::subscription::{
    commit_prepared_subscription_lifecycle_close, prepare_subscription_lifecycle_close,
    SubscriptionLifecycleCloseout,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryManagedLiveResourceCloseCause {
    Cancellation,
    Disposal,
    Replacement,
    Rebind,
    ReplacementRollback,
    RebindRollback,
    Abandonment,
}

impl WorthQueryManagedLiveResourceCloseCause {
    fn request(
        self,
        attachment: crate::subscription::SubscriptionConsumerAttachment,
    ) -> SubscriptionLifecycleCloseRequest {
        match self {
            Self::Cancellation => SubscriptionLifecycleCloseRequest::TerminateConsumer(attachment),
            Self::Disposal
            | Self::Replacement
            | Self::Rebind
            | Self::ReplacementRollback
            | Self::RebindRollback
            | Self::Abandonment => SubscriptionLifecycleCloseRequest::DetachConsumer(attachment),
        }
    }
}

impl WorthQueryRuntime {
    pub(crate) fn close_managed_live_view<T>(
        &mut self,
        view: &WorthQueryLiveView<T>,
        cause: WorthQueryManagedLiveResourceCloseCause,
    ) -> Result<SubscriptionLifecycleCloseout, WorthQueryRuntimeError> {
        let target = WorthQueryLiveArtifactTarget::from_subscription_installation(
            view.subscription_installation(),
        );
        let state = self.live_subscriptions.get(&target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingLiveSubscription(view.name().to_string())
        })?;
        if state.installation != *view.subscription_installation() {
            return Err(lifecycle_close_error(
                view.name(),
                "managed handle installation does not match Query's registered live resource",
            ));
        }
        if state.read_authority_binding.is_none() {
            return Err(lifecycle_close_error(
                view.name(),
                "managed live resource is missing its admitted read-authority binding",
            ));
        }
        if self.derived_dependency_index.has_live_dependents(&target)
            || self.effect_index.has_live_dependents(&target)
        {
            return Err(lifecycle_close_error(
                view.name(),
                "managed live resource still has registered computed or effect dependents",
            ));
        }

        let close_request = cause.request(state.consumer_attachment.clone());
        let prepared = prepare_subscription_lifecycle_close(
            &self.active_subscriptions,
            &state.active_lane_handle,
            close_request,
        )
        .map_err(|error| {
            lifecycle_close_error(
                view.name(),
                format!("{}: {}", error.denial_kind().as_str(), error.message()),
            )
        })?;
        self.backend
            .close_live_view(view.name())
            .map_err(|error| lifecycle_close_error(view.name(), error.to_string()))?;
        let closeout =
            commit_prepared_subscription_lifecycle_close(&mut self.active_subscriptions, prepared);

        self.live_subscriptions.remove(&target);
        self.materialized_read_views.remove(&target);
        self.live_subscription_index.unregister(&target);
        self.unregister_installed_live_route(&target);
        Ok(closeout)
    }
}

fn lifecycle_close_error(view_name: &str, message: impl Into<String>) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage: "managed-lifecycle-close",
        message: message.into(),
    }
}
