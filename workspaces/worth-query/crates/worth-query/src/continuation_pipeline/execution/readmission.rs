use crate::application::{
    WorthQueryDeclarationBridgeBinding, WorthQueryDeclarationBridgeContinuationRequest,
    WorthQueryDeclarationBridgeRouting, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::basis_lifecycle::LowerRuntimeEvidenceAuthority;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::{WorthQueryCommitIdentity, WorthQuerySnapshotIdentity};

use crate::continuation_pipeline::readmission::{
    continuation_readmission_basis_identity,
    continuation_readmission_lower_runtime_binding_identity,
    continuation_readmission_source_basis_identity, WorthQueryPreparedContinuationAuthorityWitness,
    WorthQueryPreparedContinuationBasisKind, WorthQueryPreparedContinuationBasisWitness,
    WorthQueryPreparedContinuationDriftKind, WorthQueryPreparedContinuationExecutionReadmission,
    WorthQueryPreparedContinuationFreshnessPosture,
};
use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionReadmissionStop,
    WorthQueryContinuationExecutionReadmissionStopKind, WorthQueryPreparedContinuation,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryPreparedContinuationCurrentReadmissionEvidence {
    authority: LowerRuntimeEvidenceAuthority,
    basis_identity: WorthQueryEvidenceIdentity,
    lower_runtime_binding_identity: Option<WorthQueryEvidenceIdentity>,
    freshness_posture: WorthQueryPreparedContinuationFreshnessPosture,
    drift_kind: Option<WorthQueryPreparedContinuationDriftKind>,
    evidence_digest: String,
}

impl WorthQueryPreparedContinuationCurrentReadmissionEvidence {
    fn new(
        authority: LowerRuntimeEvidenceAuthority,
        basis_identity: WorthQueryEvidenceIdentity,
        lower_runtime_binding_identity: Option<WorthQueryEvidenceIdentity>,
        freshness_posture: WorthQueryPreparedContinuationFreshnessPosture,
        drift_kind: Option<WorthQueryPreparedContinuationDriftKind>,
        evidence_digest: String,
    ) -> Self {
        Self {
            authority,
            basis_identity,
            lower_runtime_binding_identity,
            freshness_posture,
            drift_kind,
            evidence_digest,
        }
    }
}

pub(crate) fn prepared_execution_readmission_from_routing<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    bridge_request: &WorthQueryDeclarationBridgeContinuationRequest,
    routing: &WorthQueryDeclarationBridgeRouting<D, I>,
    required_capability_families: Vec<crate::application::WorthQueryCapabilityFamily>,
) -> WorthQueryPreparedContinuationExecutionReadmission {
    let basis_witness = basis_witness_from_binding(bridge_request, routing.binding());
    let authority_witness = authority_witness_from_binding(routing.binding());
    WorthQueryPreparedContinuationExecutionReadmission::new(
        basis_witness,
        authority_witness,
        WorthQueryPreparedContinuationFreshnessPosture::Stable,
        None,
        required_capability_families,
    )
}

pub(crate) fn current_readmission_evidence_from_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    prepared: &WorthQueryPreparedContinuation<D, I>,
) -> Result<WorthQueryPreparedContinuationCurrentReadmissionEvidence, IdentityDerivationDenial> {
    let retained = prepared.execution_readmission();
    let observation = handle
        .operating_context()
        .continuation_execution_readmission_observation(retained, handle.support_snapshot());
    Ok(
        WorthQueryPreparedContinuationCurrentReadmissionEvidence::new(
            observation.authority(),
            observation.basis_identity().clone(),
            observation.lower_runtime_binding_identity().cloned(),
            observation.freshness_posture(),
            observation.drift_kind(),
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::ContinuationExecutionReadmissionEvidence,
            )
            .field_value(
                WorthQueryEvidenceTag::new("handle"),
                handle.handle_identity_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("support_snapshot"),
                handle.support_snapshot().snapshot_digest(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis"),
                observation.basis_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("lower_runtime_binding"),
                observation.lower_runtime_binding_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("freshness"),
                observation.freshness_posture().as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("drift"),
                observation.drift_kind().map(|kind| kind.as_str()),
            )
            .seal()?
            .as_str()
            .to_string(),
        ),
    )
}

pub(crate) fn validate_execution_readmission<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    prepared: &WorthQueryPreparedContinuation<D, I>,
    current: &WorthQueryPreparedContinuationCurrentReadmissionEvidence,
) -> Result<(), WorthQueryContinuationExecutionReadmissionStop> {
    if current.freshness_posture == WorthQueryPreparedContinuationFreshnessPosture::Stale {
        return Err(WorthQueryContinuationExecutionReadmissionStop::new(
            WorthQueryContinuationExecutionReadmissionStopKind::StaleBasis,
            "the retained continuation basis evidence is stale at execution time".to_string(),
        ));
    }

    if let Some(drift_kind) = current.drift_kind {
        return Err(match drift_kind {
            WorthQueryPreparedContinuationDriftKind::AsyncRequest => {
                WorthQueryContinuationExecutionReadmissionStop::new(
                    WorthQueryContinuationExecutionReadmissionStopKind::AsyncRequestDrift,
                    "the retained continuation async request identity drifted before execution"
                        .to_string(),
                )
            }
            WorthQueryPreparedContinuationDriftKind::Replay => {
                WorthQueryContinuationExecutionReadmissionStop::new(
                    WorthQueryContinuationExecutionReadmissionStopKind::ReplayDrift,
                    "the retained continuation replay identity drifted before execution"
                        .to_string(),
                )
            }
            WorthQueryPreparedContinuationDriftKind::Remask => {
                WorthQueryContinuationExecutionReadmissionStop::new(
                    WorthQueryContinuationExecutionReadmissionStopKind::PolicyRemaskDrift,
                    "the retained continuation was remasked before execution".to_string(),
                )
            }
            WorthQueryPreparedContinuationDriftKind::PreviewCrossedResidue => {
                WorthQueryContinuationExecutionReadmissionStop::new(
                    WorthQueryContinuationExecutionReadmissionStopKind::PreviewCrossedResidue,
                    "the retained continuation crossed preview residue before execution"
                        .to_string(),
                )
            }
            WorthQueryPreparedContinuationDriftKind::StaleCompletion => {
                WorthQueryContinuationExecutionReadmissionStop::new(
                    WorthQueryContinuationExecutionReadmissionStopKind::StaleCompletion,
                    "the retained continuation completion state is stale at execution time"
                        .to_string(),
                )
            }
        });
    }

    let retained = prepared.execution_readmission();
    let witness = retained.basis_witness();
    if &current.basis_identity != witness.basis_identity() {
        return Err(
            WorthQueryContinuationExecutionReadmissionStop::new(
                WorthQueryContinuationExecutionReadmissionStopKind::BasisMismatch,
                format!(
                    "the current lower-runtime basis evidence no longer matches retained continuation basis {}",
                    witness.basis_identity_digest()
                ),
            ),
        );
    }

    if current.lower_runtime_binding_identity.as_ref()
        != witness.expected_lower_runtime_binding_identity()
    {
        return Err(WorthQueryContinuationExecutionReadmissionStop::new(
            WorthQueryContinuationExecutionReadmissionStopKind::LowerBindingMismatch,
            "the current lower-runtime binding no longer matches the retained continuation binding"
                .to_string(),
        ));
    }

    let retained_authority = lower_runtime_authority_from_witness(retained.authority_witness());
    if current.authority != retained_authority {
        return Err(
            WorthQueryContinuationExecutionReadmissionStop::new(
                WorthQueryContinuationExecutionReadmissionStopKind::AuthorityMismatch,
                format!(
                    "the current lower-runtime authority {} no longer matches retained continuation authority {}",
                    current.authority.as_str(),
                    retained_authority.as_str()
                ),
            ),
        );
    }

    Ok(())
}

fn basis_witness_from_binding(
    bridge_request: &WorthQueryDeclarationBridgeContinuationRequest,
    binding: &WorthQueryDeclarationBridgeBinding,
) -> WorthQueryPreparedContinuationBasisWitness {
    match binding {
        WorthQueryDeclarationBridgeBinding::RuntimeRoute(request) => {
            let commit_identity = bridge_commit_evidence_identity(request.commit_identity());
            WorthQueryPreparedContinuationBasisWitness::new(
                WorthQueryPreparedContinuationBasisKind::Current,
                commit_identity.clone(),
                Some(commit_identity),
                None,
            )
        }
        WorthQueryDeclarationBridgeBinding::TruthView(request) => {
            let kind = match bridge_request.truth_context() {
                crate::application::WorthQueryDeclarationBridgeTruthContext::Current => {
                    WorthQueryPreparedContinuationBasisKind::Current
                }
                crate::application::WorthQueryDeclarationBridgeTruthContext::Historical => {
                    WorthQueryPreparedContinuationBasisKind::Historical
                }
                crate::application::WorthQueryDeclarationBridgeTruthContext::Preview => {
                    WorthQueryPreparedContinuationBasisKind::PreviewDerived
                }
            };
            let selector_identity =
                continuation_readmission_basis_identity(kind, request.selector().digest());
            WorthQueryPreparedContinuationBasisWitness::new(
                kind,
                selector_identity.clone(),
                Some(continuation_readmission_lower_runtime_binding_identity(
                    request.selector().digest(),
                )),
                request
                    .selector()
                    .snapshot_identity()
                    .map(bridge_snapshot_evidence_identity)
                    .or_else(|| {
                        request
                            .selector()
                            .commit_identity()
                            .map(bridge_commit_evidence_identity)
                    }),
            )
        }
        WorthQueryDeclarationBridgeBinding::PreviewSession(request) => {
            let declaration = request.declaration();
            WorthQueryPreparedContinuationBasisWitness::new(
                WorthQueryPreparedContinuationBasisKind::PreviewDerived,
                continuation_readmission_basis_identity(
                    WorthQueryPreparedContinuationBasisKind::PreviewDerived,
                    declaration.digest(),
                ),
                Some(continuation_readmission_lower_runtime_binding_identity(
                    declaration.truth_view_basis_digest(),
                )),
                Some(continuation_readmission_source_basis_identity(
                    declaration.truth_view_basis_digest(),
                )),
            )
        }
        WorthQueryDeclarationBridgeBinding::PreviewPromotion(binding) => {
            WorthQueryPreparedContinuationBasisWitness::new(
                WorthQueryPreparedContinuationBasisKind::PreviewDerived,
                continuation_readmission_basis_identity(
                    WorthQueryPreparedContinuationBasisKind::PreviewDerived,
                    binding.promotion_continuation_digest(),
                ),
                Some(continuation_readmission_lower_runtime_binding_identity(
                    binding.preview_basis_digest(),
                )),
                Some(continuation_readmission_source_basis_identity(
                    binding.declaration_digest(),
                )),
            )
        }
        WorthQueryDeclarationBridgeBinding::SubscriptionPreparation(request) => {
            let basis_kind = basis_kind_for_truth_context(bridge_request.truth_context());
            WorthQueryPreparedContinuationBasisWitness::new(
                basis_kind,
                continuation_readmission_basis_identity(basis_kind, request.authority_digest()),
                Some(continuation_readmission_lower_runtime_binding_identity(
                    request.authority_digest(),
                )),
                request
                    .child_basis_digests()
                    .first()
                    .map(|digest| continuation_readmission_source_basis_identity(digest.as_ref())),
            )
        }
        WorthQueryDeclarationBridgeBinding::WritebackPreparation(request) => {
            let basis_kind = basis_kind_for_truth_context(bridge_request.truth_context());
            WorthQueryPreparedContinuationBasisWitness::new(
                basis_kind,
                continuation_readmission_basis_identity(basis_kind, request.causality().digest()),
                Some(continuation_readmission_lower_runtime_binding_identity(
                    request.declaration().strategy_descriptor_digest(),
                )),
                Some(continuation_readmission_source_basis_identity(
                    request.effect_intent().digest(),
                )),
            )
        }
    }
}

fn bridge_commit_evidence_identity(
    identity: &worth_runtime_bridge::facade::TruthCommitIdentity,
) -> WorthQueryEvidenceIdentity {
    let commit_id = identity
        .relational_commit_id()
        .expect("continuation readmission commit identity must carry relational payload");
    WorthQueryCommitIdentity::from_relational_commit_id(commit_id).evidence_identity()
}

fn bridge_snapshot_evidence_identity(
    identity: &worth_runtime_bridge::facade::TruthSnapshotIdentity,
) -> WorthQueryEvidenceIdentity {
    let parts = identity
        .relational_snapshot_parts()
        .expect("continuation readmission snapshot identity must carry relational payload");
    WorthQuerySnapshotIdentity::from_relational_snapshot(parts).evidence_identity()
}

fn authority_witness_from_binding(
    binding: &WorthQueryDeclarationBridgeBinding,
) -> WorthQueryPreparedContinuationAuthorityWitness {
    match binding {
        WorthQueryDeclarationBridgeBinding::RuntimeRoute(_) => {
            WorthQueryPreparedContinuationAuthorityWitness::Runtime
        }
        WorthQueryDeclarationBridgeBinding::TruthView(_) => {
            WorthQueryPreparedContinuationAuthorityWitness::RelationalFacade
        }
        WorthQueryDeclarationBridgeBinding::PreviewSession(_)
        | WorthQueryDeclarationBridgeBinding::PreviewPromotion(_)
        | WorthQueryDeclarationBridgeBinding::SubscriptionPreparation(_)
        | WorthQueryDeclarationBridgeBinding::WritebackPreparation(_) => {
            WorthQueryPreparedContinuationAuthorityWitness::RuntimeBridgeFacade
        }
    }
}

fn basis_kind_for_truth_context(
    truth_context: crate::application::WorthQueryDeclarationBridgeTruthContext,
) -> WorthQueryPreparedContinuationBasisKind {
    match truth_context {
        crate::application::WorthQueryDeclarationBridgeTruthContext::Current => {
            WorthQueryPreparedContinuationBasisKind::Current
        }
        crate::application::WorthQueryDeclarationBridgeTruthContext::Historical => {
            WorthQueryPreparedContinuationBasisKind::Historical
        }
        crate::application::WorthQueryDeclarationBridgeTruthContext::Preview => {
            WorthQueryPreparedContinuationBasisKind::PreviewDerived
        }
    }
}
pub(super) fn lower_runtime_authority_from_witness(
    witness: WorthQueryPreparedContinuationAuthorityWitness,
) -> LowerRuntimeEvidenceAuthority {
    match witness {
        WorthQueryPreparedContinuationAuthorityWitness::Runtime => {
            LowerRuntimeEvidenceAuthority::Runtime
        }
        WorthQueryPreparedContinuationAuthorityWitness::RuntimeBridgeFacade => {
            LowerRuntimeEvidenceAuthority::RuntimeBridgeFacade
        }
        WorthQueryPreparedContinuationAuthorityWitness::RelationalFacade => {
            LowerRuntimeEvidenceAuthority::RelationalFacade
        }
        WorthQueryPreparedContinuationAuthorityWitness::SignalFacade => {
            LowerRuntimeEvidenceAuthority::SignalFacade
        }
    }
}
