use worth_query::facade::runtime;

use super::{
    WorthUiPresentationAsyncDeclaration, WorthUiPresentationAsyncObservation,
    WorthUiPresentationAsyncPosture,
};

type PresentationLiveView = runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape>;

#[path = "runtime_bridge/schema.rs"]
mod schema;
use schema::{presentation_live_request, presentation_schema_view, presentation_view_name};
#[path = "runtime_bridge/completion_progress.rs"]
mod completion_progress;
pub(super) use completion_progress::WorthUiPresentationCompletionProgress;
#[path = "runtime_bridge/owned_source.rs"]
mod owned_source;
pub(super) use owned_source::installed_presentation_owned_async_source;
pub(crate) use owned_source::presentation_owned_async_source_declaration;
#[path = "runtime_bridge/request_admission.rs"]
mod request_admission;

pub struct WorthUiPresentationRuntimeAdmission {
    request: worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
    effects_indeterminate_issuer:
        worth_runtime_bridge::facade::BridgeOwnedAsyncEffectsIndeterminateIssuer,
    view: PresentationLiveView,
}

pub struct WorthUiPresentationCompletionAdvance {
    observation: WorthUiPresentationAsyncObservation,
}

#[derive(Debug)]
pub enum WorthUiPresentationCompletionDenial {
    QueryOwned(Box<runtime::WorthQueryOwnedAsyncRuntimeDenial>),
    QueryTransition(Box<runtime::WorthQueryAsyncSourceBindingError>),
    Observation(Box<WorthUiPresentationRuntimeAdmissionDenial>),
}

#[derive(Debug)]
pub(crate) enum WorthUiPresentationRuntimeAdmissionDenial {
    QueryOwned(Box<runtime::WorthQueryOwnedAsyncRuntimeDenial>),
    QueryLive(Box<runtime::WorthQueryRuntimeError>),
    MissingAsyncResultState,
    MissingSemanticRuntime,
    CleanupRequired {
        cause: Box<WorthUiPresentationRuntimeAdmissionDenial>,
        recovery: Box<WorthUiPresentationRuntimeCleanup>,
        last_denial: Box<WorthUiPresentationRuntimeCleanupDenial>,
    },
}

#[derive(Debug)]
pub(crate) enum WorthUiPresentationRuntimeCleanupDenial {
    Query(runtime::WorthQueryOwnedAsyncRuntimeDenial),
}

pub(crate) struct WorthUiPresentationRuntimeCleanup {
    request: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission>,
    request_retired: bool,
}

impl WorthUiPresentationRuntimeAdmission {
    pub(super) fn admit_in_workspace(
        workspace: &mut runtime::WorthQueryWorkspace,
        declaration: WorthUiPresentationAsyncDeclaration,
        query_declaration: &runtime::WorthQueryInstalledOwnedAsyncDeclaration,
        product: &runtime::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthUiPresentationRuntimeAdmissionDenial> {
        let request_admission = match request_admission::admit_presentation_owned_request(
            workspace,
            query_declaration,
            product,
        ) {
            Ok(request) => request,
            Err(denial) => {
                return Err(cleanup_after_admission_failure(
                    workspace,
                    None,
                    WorthUiPresentationRuntimeAdmissionDenial::QueryOwned(Box::new(denial)),
                ));
            }
        };
        let effects_indeterminate_issuer = request_admission.effects_indeterminate_issuer();
        let view = match workspace.declare_bridge_async_live_view_with_typed_identity(
            presentation_view_name(&declaration),
            presentation_live_request(),
            presentation_schema_view(),
            declaration.request_identity().request_identity(),
            request_admission.request(),
        ) {
            Ok(view) => view,
            Err(denial) => {
                return Err(cleanup_after_admission_failure(
                    workspace,
                    Some(request_admission),
                    WorthUiPresentationRuntimeAdmissionDenial::QueryLive(Box::new(denial)),
                ));
            }
        };
        Ok(Self {
            request: request_admission,
            effects_indeterminate_issuer,
            view,
        })
    }

    pub(crate) fn admit_transitions(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
        ordering: &worth_runtime_bridge::facade::BridgeMixedCauseOrdering,
    ) -> Result<
        runtime::WorthQueryAsyncResultTransitionBatch,
        runtime::WorthQueryAsyncSourceBindingError,
    > {
        workspace.admit_bridge_async_result_transitions(&self.view, ordering)
    }

    pub(super) fn admit_supersession(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
        displacing: &Self,
    ) -> Result<(), WorthUiPresentationCompletionDenial> {
        let _supersession = workspace
            .supersede_owned_bridge_async_live_view(&self.view, &self.request, &displacing.request)
            .map_err(|error| {
                WorthUiPresentationCompletionDenial::QueryTransition(Box::new(error))
            })?;
        Ok(())
    }

    pub(super) fn admit_denial_before_effects(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<(), WorthUiPresentationCompletionDenial> {
        let _denial = workspace
            .deny_owned_bridge_async_live_view(&self.view, &self.request)
            .map_err(|error| {
                WorthUiPresentationCompletionDenial::QueryTransition(Box::new(error))
            })?;
        Ok(())
    }

    pub(super) fn admit_cancellation_before_effects(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<(), WorthUiPresentationCompletionDenial> {
        let _cancellation = workspace
            .cancel_owned_bridge_async_live_view(&self.view, &self.request)
            .map_err(|error| {
                WorthUiPresentationCompletionDenial::QueryTransition(Box::new(error))
            })?;
        Ok(())
    }

    pub(super) fn close_query_live_view(
        &self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<runtime::WorthQueryLiveViewCloseReceipt, WorthUiPresentationRuntimeAdmissionDenial>
    {
        workspace
            .retire_owned_bridge_async_request(&self.request)
            .map_err(|error| {
                WorthUiPresentationRuntimeAdmissionDenial::QueryOwned(Box::new(error))
            })?;
        workspace
            .close_owned_bridge_async_live_view(&self.view)
            .map_err(|error| WorthUiPresentationRuntimeAdmissionDenial::QueryLive(Box::new(error)))
    }

    pub fn observation(
        &self,
        workspace: &runtime::WorthQueryWorkspace,
    ) -> Result<WorthUiPresentationAsyncObservation, WorthUiPresentationRuntimeAdmissionDenial>
    {
        let posture = workspace
            .state_live(&self.view)
            .map_err(|error| WorthUiPresentationRuntimeAdmissionDenial::QueryLive(Box::new(error)))?
            .async_result_state()
            .ok_or(WorthUiPresentationRuntimeAdmissionDenial::MissingAsyncResultState)?
            .kind();
        let graph = workspace
            .owned_async_runtime_topology()
            .ok_or(WorthUiPresentationRuntimeAdmissionDenial::MissingSemanticRuntime)?;
        Ok(WorthUiPresentationAsyncObservation::new(
            WorthUiPresentationAsyncPosture::from_query(posture),
            graph.signal_graph_instance(),
        ))
    }
}

fn cleanup_after_admission_failure(
    workspace: &mut runtime::WorthQueryWorkspace,
    request: Option<worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission>,
    denial: WorthUiPresentationRuntimeAdmissionDenial,
) -> WorthUiPresentationRuntimeAdmissionDenial {
    let mut recovery = WorthUiPresentationRuntimeCleanup {
        request,
        request_retired: false,
    };
    match recovery.resume(workspace) {
        Ok(()) => denial,
        Err(last_denial) => WorthUiPresentationRuntimeAdmissionDenial::CleanupRequired {
            cause: Box::new(denial),
            recovery: Box::new(recovery),
            last_denial: Box::new(last_denial),
        },
    }
}

impl std::fmt::Debug for WorthUiPresentationRuntimeCleanup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthUiPresentationRuntimeCleanup")
            .field("request_retained", &self.request.is_some())
            .field("request_retired", &self.request_retired)
            .finish()
    }
}

impl WorthUiPresentationRuntimeCleanup {
    pub(super) fn resume(
        &mut self,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<(), WorthUiPresentationRuntimeCleanupDenial> {
        if !self.request_retired {
            if let Some(request) = self.request.as_ref() {
                workspace
                    .retire_owned_bridge_async_request(request)
                    .map_err(WorthUiPresentationRuntimeCleanupDenial::Query)?;
            }
            self.request_retired = true;
        }
        Ok(())
    }
}

impl WorthUiPresentationRuntimeAdmissionDenial {
    pub(super) fn into_cleanup_required(
        self,
    ) -> Result<
        (
            WorthUiPresentationRuntimeCleanup,
            WorthUiPresentationRuntimeAdmissionDenial,
            WorthUiPresentationRuntimeCleanupDenial,
        ),
        Self,
    > {
        match self {
            Self::CleanupRequired {
                cause,
                recovery,
                last_denial,
            } => Ok((*recovery, *cause, *last_denial)),
            denial => Err(denial),
        }
    }
}

impl std::fmt::Display for WorthUiPresentationCompletionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueryOwned(denial) => write!(formatter, "Query-owned completion: {denial:?}"),
            Self::QueryTransition(denial) => {
                write!(formatter, "Query completion transition: {denial:?}")
            }
            Self::Observation(denial) => write!(formatter, "completion observation: {denial}"),
        }
    }
}

impl std::fmt::Display for WorthUiPresentationRuntimeAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QueryOwned(denial) => write!(formatter, "Query-owned admission: {denial:?}"),
            Self::QueryLive(denial) => write!(formatter, "Query live-view admission: {denial:?}"),
            Self::MissingAsyncResultState => formatter.write_str("missing async result state"),
            Self::MissingSemanticRuntime => formatter.write_str("missing semantic runtime"),
            Self::CleanupRequired {
                cause,
                recovery,
                last_denial,
            } => write!(
                formatter,
                "admission cleanup required after {cause}; request retained: {}, last denial: {last_denial}",
                recovery.request.is_some(),
            ),
        }
    }
}

impl std::fmt::Display for WorthUiPresentationRuntimeCleanupDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(denial) => write!(formatter, "Query cleanup: {denial:?}"),
        }
    }
}
