use std::marker::PhantomData;

use crate::application::{
    WorthQueryBridgeContinuationAuthority, WorthQueryCapabilityFamily, WorthQueryConfig,
    WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationBridgeContinuationMode,
    WorthQueryDeclarationBridgeContinuationRequest, WorthQueryDeclarationBridgeTruthContext,
    WorthQueryDeclarationCanonicalEntry, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityContract,
    WorthQueryDeclarationRouteContract, WorthQueryDeclarationSignalCompatibilityContract,
    WorthQueryDeclarationSignalCompatibilityInput, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryNeighborhoodCapableGrouping,
    WorthQuerySignalCompatiblePosture,
};

use crate::signal_compatibility_orchestration::{
    WorthQuerySignalCompatibilityOrchestration, WorthQuerySignalCompatibilityOrchestrationClass,
    WorthQuerySignalCompatibilityOrchestrationInput,
    WorthQuerySignalCompatibilityOrchestrationOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SignalDomain;

impl WorthQueryDomainEntryMarker for SignalDomain {
    fn domain_key(&self) -> &'static str {
        "test.signal.orchestration.domain"
    }

    fn display_name(&self) -> &'static str {
        "SignalOrchestrationDomain"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[WorthQueryCapabilityFamily::QueryComposition]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SignalWorld(pub(super) &'static str);

impl WorthQueryDomainOperatingContext<SignalDomain> for SignalWorld {
    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[
            WorthQueryCapabilityFamily::HistoricalEvaluation,
            WorthQueryCapabilityFamily::WorkflowOrchestration,
            WorthQueryCapabilityFamily::PreviewSession,
            WorthQueryCapabilityFamily::QueryComposition,
        ]
    }

    fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        &[
            WorthQueryConfigSectionFamily::Query,
            WorthQueryConfigSectionFamily::Relational,
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQueryConfigSectionFamily::Signal,
        ]
    }

    fn context_identity(
        &self,
    ) -> crate::application::WorthQueryDomainOperatingContextIdentityDeclaration {
        let value = { format!("signal-world-{}", self.0) };
        crate::application::WorthQueryDomainOperatingContextIdentityDeclaration::single(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SignalFamily;

impl WorthQueryDeclarationFamilyMarker<SignalDomain> for SignalFamily {
    type PrimaryAuthority = WorthQueryBridgeContinuationAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "SignalFamily"
    }

    fn aspect_contract() -> WorthQueryDeclarationAspectContract {
        WorthQueryDeclarationAspectContract::from_slices(
            &[
                "selection.face",
                "selection.active_face",
                "signal.dependency.runtime_inputs",
            ],
            &["selection.neighborhood.local_topology"],
            &["signal.preview.surface"],
            &["signal.private_authority"],
            &["signal.conflicting_dependency"],
        )
    }

    fn aspect_coverage() -> WorthQueryDeclarationAspectCoverage {
        WorthQueryDeclarationAspectCoverage::from_slices(
            &[
                "selection.face",
                "selection.active_face",
                "signal.dependency.runtime_inputs",
                "selection.neighborhood.local_topology",
                "signal.preview.surface",
            ],
            &[],
            &[],
        )
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }

    fn route_contract() -> WorthQueryDeclarationRouteContract {
        WorthQueryDeclarationRouteContract::bridge_only()
    }

    fn bridge_continuation_contract(
    ) -> Option<crate::application::WorthQueryDeclarationBridgeContinuationContract> {
        Some(
            crate::application::WorthQueryDeclarationBridgeContinuationContract::runtime_route_current(),
        )
    }

    fn signal_compatibility_contract() -> Option<WorthQueryDeclarationSignalCompatibilityContract> {
        Some(
            WorthQueryDeclarationSignalCompatibilityContract::runtime_derived_execution()
                .with_aspects(
                    WorthQueryDeclarationAspectContract::from_slices(
                        &["selection.active_face", "signal.dependency.runtime_inputs"],
                        &["selection.neighborhood.local_topology"],
                        &[],
                        &["signal.private_authority"],
                        &["signal.conflicting_dependency"],
                    ),
                    WorthQueryDeclarationAspectContract::from_slices(
                        &["signal.produced.derived_face_preview"],
                        &["signal.produced.material_projection"],
                        &["signal.produced.preview.surface"],
                        &[],
                        &[],
                    ),
                ),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SignalInput {
    id: &'static str,
    _marker: PhantomData<SignalFamily>,
}

impl SignalInput {
    pub(super) fn new(id: &'static str) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl WorthQueryDeclarationInput<SignalDomain> for SignalInput {
    type Family = SignalFamily;

    fn canonical_declaration_entries(&self) -> Vec<WorthQueryDeclarationCanonicalEntry> {
        vec![WorthQueryDeclarationCanonicalEntry::text("id", self.id)]
    }
}

pub(super) fn admitted_handle(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, SignalWorld> {
    admitted_handle_with_config(
        request_execution,
        world,
        WorthQueryConfig::runtime_backed_default(),
    )
}

fn admitted_handle_with_config(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &'static str,
    _config: WorthQueryConfig,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, SignalWorld> {
    installed_signal_context(request_execution, SignalWorld(world))
}

pub(super) fn installed_signal_context<C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: C,
) -> crate::application::WorthQueryInstalledDomainDeclarationContext<SignalDomain, C>
where
    C: WorthQueryDomainOperatingContext<SignalDomain>,
{
    crate::application::domain_test_support::installed_declaration_context(
        request_execution,
        SignalDomain,
        context,
        [crate::application::domain_test_support::family::<
            SignalDomain,
            SignalFamily,
        >()],
    )
}

pub(super) fn bridge_request() -> WorthQueryDeclarationBridgeContinuationRequest {
    WorthQueryDeclarationBridgeContinuationRequest::new(
        WorthQueryDeclarationBridgeContinuationMode::RuntimeRoute,
        WorthQueryDeclarationBridgeTruthContext::Current,
    )
}

pub(super) fn envelope(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        SignalDomain,
        SignalWorld,
    >,
    id: &'static str,
) -> crate::application::WorthQueryDeclarationEnvelope<SignalDomain, SignalInput> {
    let progressed = handle
        .declare_review_and_progress(SignalInput::new(id))
        .unwrap_or_else(|_| panic!("expected progressed signal declaration"));
    handle
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("expected signal envelope"))
}

pub(super) fn orchestration_input(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        SignalDomain,
        SignalWorld,
    >,
    id: &'static str,
) -> WorthQuerySignalCompatibilityOrchestrationInput<SignalDomain, SignalInput> {
    WorthQuerySignalCompatibilityOrchestrationInput::new(
        WorthQueryDeclarationSignalCompatibilityInput::enveloped(envelope(handle, id)),
    )
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum OutcomeDigestToken {
    Compatible(String),
    Prepared {
        prepared_digest: String,
        signal_compatibility_digest: Option<String>,
    },
    Status(&'static str),
}

pub(super) fn orchestration_outcome_token(
    outcome: &WorthQuerySignalCompatibilityOrchestrationOutcome<SignalDomain, SignalInput>,
) -> OutcomeDigestToken {
    match outcome {
        WorthQuerySignalCompatibilityOrchestrationOutcome::Bound(value) => match value.class() {
            WorthQuerySignalCompatibilityOrchestrationClass::Compatible => {
                OutcomeDigestToken::Compatible(
                    value
                        .signal_execution_family()
                        .map(|family| format!("{family:?}:{:?}", value.envelope_digest()))
                        .unwrap_or_else(|| format!("compatible:{:?}", value.envelope_digest())),
                )
            }
            WorthQuerySignalCompatibilityOrchestrationClass::Prepared => match value {
                WorthQuerySignalCompatibilityOrchestration::Prepared(prepared) => {
                    OutcomeDigestToken::Prepared {
                        prepared_digest: prepared.prepared_digest().to_string(),
                        signal_compatibility_digest: prepared
                            .signal_compatibility_digest()
                            .map(str::to_string),
                    }
                }
                _ => unreachable!(),
            },
        },
        WorthQuerySignalCompatibilityOrchestrationOutcome::Ambiguous(_) => {
            OutcomeDigestToken::Status("ambiguous")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Unavailable(_) => {
            OutcomeDigestToken::Status("unavailable")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::WrongWorld(_) => {
            OutcomeDigestToken::Status("wrong_world")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::WrongHandle(_) => {
            OutcomeDigestToken::Status("wrong_handle")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::InstalledAuthorityDrift(_) => {
            OutcomeDigestToken::Status("installed_authority_drift")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Stale(_) => {
            OutcomeDigestToken::Status("stale")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::RebindRequired(_) => {
            OutcomeDigestToken::Status("rebind_required")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::MissingRequiredAspect(_) => {
            OutcomeDigestToken::Status("missing_required_aspect")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::AspectConflict(_) => {
            OutcomeDigestToken::Status("aspect_conflict")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::AuthorityMismatch(_) => {
            OutcomeDigestToken::Status("authority_mismatch")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::BasisMismatch(_) => {
            OutcomeDigestToken::Status("basis_mismatch")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Deferred(_) => {
            OutcomeDigestToken::Status("deferred")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Denied(_) => {
            OutcomeDigestToken::Status("denied")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Unsupported(_) => {
            OutcomeDigestToken::Status("unsupported")
        }
        WorthQuerySignalCompatibilityOrchestrationOutcome::Failed(_) => {
            OutcomeDigestToken::Status("failed")
        }
    }
}

pub(super) fn continuation_outcome_token(
    outcome: &crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome<
        SignalDomain,
        SignalInput,
    >,
) -> OutcomeDigestToken {
    match outcome {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            OutcomeDigestToken::Prepared {
                prepared_digest: prepared.prepared_digest().to_string(),
                signal_compatibility_digest: prepared
                    .signal_compatibility_digest()
                    .map(str::to_string),
            }
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Ambiguous(_) => {
            OutcomeDigestToken::Status("ambiguous")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Unavailable(_) => {
            OutcomeDigestToken::Status("unavailable")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::WrongWorld(_) => {
            OutcomeDigestToken::Status("wrong_world")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::WrongHandle(_) => {
            OutcomeDigestToken::Status("wrong_handle")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::InstalledAuthorityDrift(
            _,
        ) => OutcomeDigestToken::Status("installed_authority_drift"),
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Stale(_) => {
            OutcomeDigestToken::Status("stale")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::RebindRequired(_) => {
            OutcomeDigestToken::Status("rebind_required")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::AuthorityMismatch(
            _,
        ) => OutcomeDigestToken::Status("authority_mismatch"),
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::BasisMismatch(_) => {
            OutcomeDigestToken::Status("basis_mismatch")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Unsupported(_) => {
            OutcomeDigestToken::Status("unsupported")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Deferred(_) => {
            OutcomeDigestToken::Status("deferred")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Denied(_) => {
            OutcomeDigestToken::Status("denied")
        }
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Failed(_) => {
            OutcomeDigestToken::Status("failed")
        }
    }
}
