use crate::identity::CollectionPlanDigest;
use crate::live::LiveQueryPlan;
use crate::preview::binding::{PreviewBindingFailureClass, PreviewSessionPlanBinding};
#[cfg(test)]
use crate::preview::execution::PreviewExecutionError;
use crate::preview::live::{
    PreviewLiveAdmissionReport, PreviewLiveCounters, PreviewLiveDriftDenied,
    PreviewLiveDriftOutcome, PreviewLiveError, PreviewLiveFailureClass, PreviewLiveMaintained,
    PreviewLiveRebindArtifact, PreviewLiveSessionPlanBinding,
};
use crate::preview::session_context::PreviewSessionQueryContext;
use crate::preview::workflow_context_identity;
use crate::preview::{
    admit_scoped_preview_live_session_plan,
    admit_scoped_preview_session_plan_binding_from_preview_binding,
    bind_preflight_to_preview_session, ScopedPreviewLiveSessionPlanBinding,
};
#[cfg(test)]
use crate::preview::{execute_scoped_preview_live_session_plan, PreviewLiveExecutionEnvelope};
use worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind;

pub(crate) fn admit_preview_live_session_plan_component(
    preview_binding: PreviewSessionPlanBinding,
    live_plan: LiveQueryPlan,
) -> Result<PreviewLiveSessionPlanBinding, PreviewLiveError> {
    let preview_query = preview_binding.preflight().plan().query();
    let live_descriptor = live_plan.descriptor();

    if live_descriptor.query_digest() != preview_query.validated_query_digest() {
        return Err(PreviewLiveError {
            failure_class: PreviewLiveFailureClass::PreviewLiveQueryDigestMismatch,
            message: "preview-live admission requires the same validated query digest across preview and live proofs",
            counters: PreviewLiveCounters {
                preview_live_broad_fallback_denial_count: 1,
                ..PreviewLiveCounters::default()
            },
        });
    }

    if live_descriptor.plan_digest() != preview_query.plan_digest() {
        return Err(PreviewLiveError {
            failure_class: PreviewLiveFailureClass::PreviewLivePlanDigestMismatch,
            message: "preview-live admission requires the same planned query digest across preview and live proofs",
            counters: PreviewLiveCounters {
                preview_live_broad_fallback_denial_count: 1,
                ..PreviewLiveCounters::default()
            },
        });
    }

    if live_plan.start_basis().basis().proof().digest().as_str()
        != preview_binding
            .preflight()
            .basis()
            .proof()
            .digest()
            .as_str()
    {
        return Err(PreviewLiveError {
            failure_class: PreviewLiveFailureClass::PreviewLiveBasisMismatch,
            message: "preview-live admission requires the live plan to derive from the same authoritative basis as the preview preflight",
            counters: PreviewLiveCounters {
                preview_live_broad_fallback_denial_count: 1,
                ..PreviewLiveCounters::default()
            },
        });
    }

    let preview_collection_digest = preview_binding
        .preflight()
        .plan()
        .collection()
        .map(|collection| collection.digest().as_str());
    let live_collection_digest = live_descriptor
        .collection_digest()
        .map(CollectionPlanDigest::as_str);

    if preview_collection_digest != live_collection_digest {
        return Err(PreviewLiveError {
            failure_class: PreviewLiveFailureClass::PreviewLiveCollectionDigestMismatch,
            message: "preview-live admission requires matching collection planning identity across preview and live proofs",
            counters: PreviewLiveCounters {
                preview_live_broad_fallback_denial_count: 1,
                ..PreviewLiveCounters::default()
            },
        });
    }

    let report = PreviewLiveAdmissionReport {
        digest: workflow_context_identity::compose_preview_live_admission_digest(
            preview_binding.basis().binding_tuple().digest(),
            live_plan.subscription_digest().as_str(),
            live_descriptor.family().as_str(),
        )
        .map_err(|cause| {
            PreviewLiveError::identity_refused(cause, PreviewLiveCounters::default())
        })?,
        preview_binding_digest: preview_binding.basis().binding_tuple().digest().to_string(),
        live_subscription_digest: live_plan.subscription_digest().as_str().to_string(),
        live_family: live_descriptor.family().as_str().to_string(),
        counters: PreviewLiveCounters {
            preview_live_admission_count: 1,
            ..PreviewLiveCounters::default()
        },
    };

    Ok(PreviewLiveSessionPlanBinding {
        preview_binding,
        live_plan,
        report,
    })
}

#[cfg(test)]
pub(crate) fn preview_live_execution_counters(
    preview_live: &PreviewLiveSessionPlanBinding,
) -> Result<PreviewLiveCounters, PreviewExecutionError> {
    let mut counters = preview_live.report().counters().clone();
    counters.preview_live_execution_count = 1;
    Ok(counters)
}

#[cfg(test)]
pub(crate) fn admit_preview_live_session_plan(
    preview_binding: PreviewSessionPlanBinding,
    live_plan: LiveQueryPlan,
) -> Result<ScopedPreviewLiveSessionPlanBinding, PreviewLiveError> {
    let scoped_binding =
        admit_scoped_preview_session_plan_binding_from_preview_binding(preview_binding)?;
    admit_scoped_preview_live_session_plan(scoped_binding, live_plan)
}

#[cfg(test)]
pub(crate) fn execute_preview_live_session_plan(
    preview_live: &ScopedPreviewLiveSessionPlanBinding,
) -> Result<PreviewLiveExecutionEnvelope, PreviewExecutionError> {
    execute_scoped_preview_live_session_plan(preview_live)
}

pub fn assess_preview_live_drift(
    preview_live: &ScopedPreviewLiveSessionPlanBinding,
    refreshed_context: PreviewSessionQueryContext,
) -> PreviewLiveDriftOutcome {
    let mut lifecycle_counters = PreviewLiveCounters {
        preview_live_lifecycle_check_count: 1,
        ..PreviewLiveCounters::default()
    };

    if refreshed_context.lifecycle_state_kind() != BridgePreviewLifecycleStateKind::Active {
        lifecycle_counters.preview_live_drift_denial_count = 1;
        return PreviewLiveDriftOutcome::DriftDenied(PreviewLiveDriftDenied {
            prior_preview_live_digest: preview_live.scoped_live_digest().to_string(),
            lifecycle_state_kind: refreshed_context.lifecycle_state_kind(),
            error: PreviewLiveError {
                failure_class: PreviewLiveFailureClass::PreviewLiveLifecycleDrifted,
                message: "preview-live maintenance may continue only while the preview session remains active",
                counters: lifecycle_counters,
            },
        });
    }

    let rebound_binding = match bind_preflight_to_preview_session(
        preview_live
            .preview_live_component()
            .preview_binding()
            .preflight()
            .clone(),
        refreshed_context,
    ) {
        Ok(binding) => binding,
        Err(error) => {
            let failure_class = match error.failure_class() {
                PreviewBindingFailureClass::EvidenceIdentityDerivationDenied(cause) => {
                    PreviewLiveFailureClass::EvidenceIdentityDerivationDenied(*cause)
                }
                PreviewBindingFailureClass::InvalidPreviewBasis => {
                    PreviewLiveFailureClass::PreviewLiveBroadFallbackForbidden
                }
                PreviewBindingFailureClass::MissingExecutionRecordIdentity => {
                    PreviewLiveFailureClass::PreviewLiveRebindBindingRejected
                }
                PreviewBindingFailureClass::StaleOrInactivePreviewLifecycle => {
                    PreviewLiveFailureClass::PreviewLiveLifecycleDrifted
                }
                PreviewBindingFailureClass::RawBranchAliasPreviewForbidden
                | PreviewBindingFailureClass::UnsupportedPreviewQueryFamily
                | PreviewBindingFailureClass::PromotionLinkageMismatch
                | PreviewBindingFailureClass::StoreBackedRouteForbidden => {
                    PreviewLiveFailureClass::PreviewLiveBroadFallbackForbidden
                }
            };
            let mut counters = lifecycle_counters.clone();
            counters.preview_live_drift_denial_count = 1;
            if matches!(
                failure_class,
                PreviewLiveFailureClass::PreviewLiveBroadFallbackForbidden
            ) {
                counters.preview_live_broad_fallback_denial_count = 1;
            }
            return PreviewLiveDriftOutcome::DriftDenied(PreviewLiveDriftDenied {
                prior_preview_live_digest: preview_live.scoped_live_digest().to_string(),
                lifecycle_state_kind: BridgePreviewLifecycleStateKind::Active,
                error: PreviewLiveError {
                    failure_class: failure_class.clone(),
                    message: if matches!(
                        failure_class,
                        PreviewLiveFailureClass::PreviewLiveBroadFallbackForbidden
                    ) {
                        "preview-live drift handling may not recover by silently broadening or retargeting basis"
                    } else {
                        error.message()
                    },
                    counters,
                },
            });
        }
    };

    let rebound_preview_live =
        match admit_scoped_preview_session_plan_binding_from_preview_binding(rebound_binding)
            .and_then(|binding| {
                admit_scoped_preview_live_session_plan(binding, preview_live.live_plan().clone())
            }) {
            Ok(binding) => binding,
            Err(error) => {
                let mut counters = error.counters.clone();
                counters.preview_live_lifecycle_check_count += 1;
                counters.preview_live_drift_denial_count += 1;
                return PreviewLiveDriftOutcome::DriftDenied(PreviewLiveDriftDenied {
                    prior_preview_live_digest: preview_live.scoped_live_digest().to_string(),
                    lifecycle_state_kind: BridgePreviewLifecycleStateKind::Active,
                    error: PreviewLiveError {
                        failure_class: error.failure_class,
                        message: error.message,
                        counters,
                    },
                });
            }
        };

    if rebound_preview_live == *preview_live {
        return PreviewLiveDriftOutcome::Maintained(PreviewLiveMaintained {
            maintained_preview_live: rebound_preview_live,
            counters: lifecycle_counters,
        });
    }

    lifecycle_counters.preview_live_rebind_available_count = 1;
    PreviewLiveDriftOutcome::ExplicitRebindAvailable(PreviewLiveRebindArtifact {
        prior_preview_live_digest: preview_live.scoped_live_digest().to_string(),
        rebound_preview_live,
        counters: lifecycle_counters,
    })
}
