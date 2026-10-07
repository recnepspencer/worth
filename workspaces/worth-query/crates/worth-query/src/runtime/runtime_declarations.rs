use super::async_result_state::WorthQueryRuntimeAsyncResultProjection;
use super::async_source_binding::WorthQueryRuntimeAsyncSourceBinding;
use super::*;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryLiveViewHandle, WorthQueryWorkspaceError,
};

impl WorthQueryRuntime {
    pub fn declare_live_view<T>(
        &mut self,
        name: impl Into<String>,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Live)?;
        let name = name.into();
        admit_live_view_declaration_receipt(&*self.backend, &name, &request, &schema_view)?;
        let activation = self.install_live_subscription_for_request(
            &name,
            &request,
            schema_view.clone(),
            crate::subscription::QuerySubscriptionFutureSelection::ordinary(),
        )?;
        self.finish_live_view_declaration(name, request, schema_view, activation, None)
    }

    pub(crate) fn declare_live_view_from_read_binding<T>(
        &mut self,
        name: impl Into<String>,
        binding: WorthQueryReadExecutionBinding,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Live)?;
        let name = name.into();
        if self
            .live_subscriptions
            .contains_key(&WorthQueryLiveArtifactTarget::from_view_name(&name))
        {
            return Err(WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: name,
                stage: "managed-resource-name-admission",
                message: "managed live resource names must be unique within a workspace"
                    .to_string(),
            });
        }
        let request = binding
            .read_family()
            .read_graph()
            .declarative_request()
            .clone();
        let schema_view = binding.read_family().read_graph().schema_view().clone();
        admit_live_view_declaration_receipt(&*self.backend, &name, &request, &schema_view)?;
        let activation = self.install_live_subscription_for_read_binding(&name, binding)?;
        self.finish_live_view_declaration(name, request, schema_view, activation, None)
    }

    pub(super) fn finish_live_view_declaration<T>(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        activation: WorthQueryRuntimeLiveSubscriptionActivation,
        async_source_binding: Option<WorthQueryRuntimeAsyncSourceBinding>,
    ) -> Result<WorthQueryLiveView<T>, WorthQueryRuntimeError> {
        let handle = declare_live_view_source_handle(
            &mut *self.backend,
            &mut self.active_subscriptions,
            &name,
            request,
            schema_view,
            &activation,
        )?;
        self.live_subscription_index.register(
            WorthQueryLiveArtifactTarget::from_subscription_installation(&activation.installation),
            &activation.request,
        );
        let future_selection = activation.active_lane_handle.future_selection();
        let pending_async_projection_digest = future_selection
            .future_selection_projection()
            .label()
            .to_string();
        let pending_async_result_state =
            future_selection.requests_completion_lifecycle().then(|| {
                WorthQueryRuntimeAsyncResultProjection::pending(&format!(
                    "async-pending:{pending_async_projection_digest}"
                ))
            });
        let basis_binding_identity = async_source_binding
            .as_ref()
            .map(WorthQueryRuntimeAsyncSourceBinding::current_basis_identity)
            .unwrap_or_else(|| activation.installation.basis_binding_identity().clone());
        let checkpoint_identity = async_source_binding
            .as_ref()
            .map(WorthQueryRuntimeAsyncSourceBinding::current_generation_identity)
            .unwrap_or_else(|| activation.active_lane_handle.checkpoint_identity().clone());
        let live_target =
            WorthQueryLiveArtifactTarget::from_subscription_installation(&activation.installation);
        self.live_subscriptions.insert(
            live_target,
            WorthQueryRuntimeLiveSubscriptionState {
                installation: activation.installation.clone(),
                active_lane_handle: activation.active_lane_handle,
                consumer_attachment: activation.consumer_attachment,
                request: activation.request,
                delivery_batches: Vec::new(),
                last_delivery: None,
                async_result_state: None,
                async_source_binding,
                remask_posture: activation.remask_posture,
                read_authority_binding: activation.read_authority_binding,
            },
        );
        if let Some(projection) = pending_async_result_state.as_ref() {
            self.project_async_result_state(
                &name,
                projection,
                &basis_binding_identity,
                &checkpoint_identity,
            )?;
        }
        Ok(WorthQueryLiveView::new(handle, activation.installation))
    }

    pub fn declare_derived_view(
        &mut self,
        view: WorthQueryDerivedView,
    ) -> Result<WorthQueryDerivedView, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Computed)?;
        self.admit_derived_view_declaration(&view)?;
        insert_derived_runtime(
            &mut self.derived_views,
            &mut self.derived_dependency_index,
            view.clone(),
            None,
        );
        Ok(view)
    }

    pub fn declare_maintained_derived_view<T>(
        &mut self,
        view: WorthQueryDerivedView,
        maintainer: impl WorthQueryDerivedViewMaintainer + 'static,
    ) -> Result<WorthQueryDerivedViewHandle<T>, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Computed)?;
        self.admit_derived_view_declaration(&view)?;
        let name = view.name().to_string();
        insert_derived_runtime(
            &mut self.derived_views,
            &mut self.derived_dependency_index,
            view,
            Some(Box::new(maintainer)),
        );
        self.initialize_maintained_derived_view_materialization(&name)?;
        Ok(WorthQueryDerivedViewHandle::new(name))
    }

    fn admit_derived_view_declaration(
        &self,
        view: &WorthQueryDerivedView,
    ) -> Result<(), WorthQueryRuntimeError> {
        let live_view_targets = self.live_subscriptions.keys().cloned().collect();
        admit_derived_view_declaration(&self.derived_views, &live_view_targets, view).map_err(
            |error| WorthQueryRuntimeError::ComputedDeclaration {
                view_name: view.name().to_string(),
                stage: "dependency-admission",
                message: error.message(),
            },
        )
    }

    pub fn declare_effect<T>(
        &mut self,
        declaration: WorthQueryEffectDeclaration,
    ) -> Result<WorthQueryEffectHandle<T>, WorthQueryRuntimeError> {
        self.reap_abandoned_managed_live_resources()?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Effect)?;
        let live_view_targets = self
            .live_subscriptions
            .values()
            .map(|state| {
                WorthQueryLiveArtifactTarget::from_subscription_installation(&state.installation)
            })
            .collect();
        let computed_view_targets = self.derived_views.keys().cloned().collect();
        admit_effect_declaration(&live_view_targets, &computed_view_targets, &declaration)?;
        let name = declaration.name().to_string();
        let target_lane = declaration.target_lane();
        insert_effect_runtime(&mut self.effects, &mut self.effect_index, declaration);
        Ok(WorthQueryEffectHandle::new(name, target_lane))
    }
}

impl WorthQueryRuntime {
    fn initialize_maintained_derived_view_materialization(
        &mut self,
        view_name: &str,
    ) -> Result<(), WorthQueryRuntimeError> {
        let target = WorthQueryDerivedMaterializationTarget::new(view_name);
        let declaration = match self.derived_views.get(&target) {
            Some(runtime) if !runtime.declaration.incremental() && runtime.maintainer.is_some() => {
                runtime.declaration.clone()
            }
            _ => return Ok(()),
        };
        let upstreams = retained_upstream_inputs_for_declaration(self, &declaration)?;
        let snapshot_identity = self.current_snapshot_identity()?;
        let refresh_identity = WorthQueryCommitIdentity::preview(
            WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("refresh_origin"),
                "derived-declaration",
            )
            .field_value(WorthQueryEvidenceTag::new("view_name"), view_name)
            .seal(),
        );
        let refresh_metadata = self
            .backend
            .declaration_initialization_metadata(&declaration)
            .map_err(WorthQueryRuntimeError::Workspace)?;
        let refresh = WorthQueryRetainedRefreshContext::from_declaration_initialization(
            refresh_identity,
            snapshot_identity,
            refresh_metadata,
        );
        let Some(runtime) = self.derived_views.get_mut(&target) else {
            return Ok(());
        };
        let Some(maintainer) = runtime.maintainer.as_mut() else {
            return Ok(());
        };
        let _ = maintainer.refresh_from_upstreams(
            &declaration,
            &refresh,
            &upstreams,
            &mut runtime.materialization,
        );
        Ok(())
    }
}

pub(super) fn admit_live_view_declaration_receipt(
    backend: &dyn WorthQueryRuntimeBackend,
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
    schema_view: &QuerySchemaView,
) -> Result<(), WorthQueryRuntimeError> {
    let admission_receipt = backend
        .admit_live_view_declaration(view_name, request, schema_view)
        .map_err(
            |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
                view_name: view_name.to_string(),
                stage: "backend-live-admission",
                message: error.to_string(),
            },
        )?;
    if let Some(message) = admission_receipt.drift_from_request(view_name, request) {
        return Err(WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "backend-live-admission-receipt",
            message,
        });
    }
    Ok(())
}

fn declare_live_view_source_handle(
    backend: &mut dyn WorthQueryRuntimeBackend,
    active_subscriptions: &mut ActiveSubscriptionRuntime,
    view_name: &str,
    request: DeclarativeLiveQueryRequest,
    schema_view: QuerySchemaView,
    activation: &WorthQueryRuntimeLiveSubscriptionActivation,
) -> Result<WorthQueryLiveViewHandle, WorthQueryRuntimeError> {
    backend
        .declare_live_view(view_name.to_string(), request, schema_view)
        .map_err(|error| {
            live_source_declaration_error(active_subscriptions, view_name, error, activation)
        })
}

fn live_source_declaration_error(
    active_subscriptions: &mut ActiveSubscriptionRuntime,
    view_name: &str,
    error: WorthQueryWorkspaceError,
    activation: &WorthQueryRuntimeLiveSubscriptionActivation,
) -> WorthQueryRuntimeError {
    let closeout_result = close_subscription_lifecycle(
        active_subscriptions,
        &activation.active_lane_handle,
        SubscriptionLifecycleCloseRequest::DetachConsumer(activation.consumer_attachment.clone()),
    );
    let closeout_message = match closeout_result {
        Ok(closeout) => format!(
            "active subscription closeout:{}:terminal:{}",
            closeout.closeout_projection().label(),
            closeout.lane_terminal()
        ),
        Err(closeout_error) => format!(
            "active subscription closeout failed:{}:{}",
            closeout_error.denial_kind().as_str(),
            closeout_error.message()
        ),
    };
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage: "source-declaration",
        message: format!("{error}; {closeout_message}"),
    }
}

fn retained_upstream_inputs_for_declaration(
    runtime: &mut WorthQueryRuntime,
    declaration: &WorthQueryDerivedView,
) -> Result<WorthQueryRetainedUpstreamInputs, WorthQueryRuntimeError> {
    let live_rows = declaration
        .upstream_live_views()
        .iter()
        .map(|view_name| {
            let installation = runtime
                .live_subscriptions
                .get(&WorthQueryLiveArtifactTarget::from_view_name(view_name))
                .map(|state| state.installation.clone())
                .ok_or_else(|| WorthQueryRuntimeError::MissingLiveView(view_name.clone()))?;
            runtime
                .execute_live_read_for_installation(installation)
                .map(|read| {
                    (
                        WorthQueryLiveArtifactTarget::from_view_name(view_name),
                        read.rows().to_vec(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let computed_rows = declaration
        .upstream_derived_views()
        .iter()
        .filter_map(|view_name| {
            let target = WorthQueryDerivedMaterializationTarget::new(view_name);
            runtime
                .derived_views
                .get(&target)
                .map(|runtime| (target, runtime.materialization.retained_rows().to_vec()))
        })
        .collect::<Vec<_>>();
    Ok(WorthQueryRetainedUpstreamInputs::from_retained_computed_rows(live_rows, computed_rows))
}
