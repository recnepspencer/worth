mod basis_projection;
mod bridge;
mod effect;
mod inspection;
mod neighbors;
mod read;
mod routing;
mod runtime;
mod write_authority;

pub(in crate::intent_admission::certification) use basis_projection::{
    certified_basis_observation_intent_fixture, certified_projection_consumption_admitted_fixture,
    certified_projection_consumption_warning_fixture,
};
pub(crate) use bridge::certification_bridge;
pub(in crate::intent_admission::certification) use effect::certified_effect_intent_fixture;
pub(in crate::intent_admission::certification) use inspection::certified_inspection_advisory_redaction_fixture;
pub(in crate::intent_admission::certification) use neighbors::{
    certified_deferred_intent_fixture, certified_unsupported_intent_fixture,
    CertifiedDeferredIntentFixture, CertifiedUnsupportedIntentFixture,
};
pub(in crate::intent_admission::certification) use read::{
    certified_read_intent_fixture, read_delegation_parity_fixture, CertifiedReadIntentFixture,
};
pub(in crate::intent_admission::certification) use routing::{
    certified_routing_intent_fixture, routing_delegation_parity_fixture,
    CertifiedRoutingIntentFixture, RoutingDelegationParityFixture,
};
pub(crate) use runtime::{
    certification_runtime, certification_runtime_with_invariant_violation_authority,
    certification_task_live_request, certification_task_schema,
};

use crate::facade::runtime::{
    WorthQueryAspectTouch, WorthQueryEffectDeclaration, WorthQueryEffectTrigger,
    WorthQueryIntentAdmissionDecision, WorthQueryIntentAdvisoryDecision,
    WorthQueryIntentDeclaration, WorthQueryWriteCommand,
};
use crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData;
use crate::intent_admission::{admit_runtime_intent_request, WorthQueryRawIntentAdmissionRequest};
use crate::memory_workspace::{WorthQueryEntityIdentity, WorthQuerySnapshotIdentity};
use crate::runtime::WorthQueryUnrefinedLiveShape;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
};

pub(super) fn certification_snapshot_identity(
    label: impl AsRef<str>,
) -> WorthQuerySnapshotIdentity {
    certification_snapshot_identity_for("certification-snapshot", label)
}

pub(super) fn certification_snapshot_identity_for(
    namespace: impl AsRef<str>,
    evidence: impl AsRef<str>,
) -> WorthQuerySnapshotIdentity {
    let snapshot_id = stable_certification_position(namespace.as_ref(), evidence.as_ref());
    let version_id =
        stable_certification_position(format!("{}:version", namespace.as_ref()), evidence.as_ref());
    WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(snapshot_id, version_id),
    )
}

pub(super) fn certification_entity_identity(label: impl AsRef<str>) -> WorthQueryEntityIdentity {
    WorthQueryEntityIdentity::from_relational_record(RelationalBridgeRecordIdentityParts::entity(
        1,
        stable_certification_position("certification-entity", label),
        0,
    ))
}

pub(super) fn identity_id_touch() -> WorthQueryAspectTouch {
    certification_aspect_field_touch("identity", "id")
}

pub(super) fn title_value_touch() -> WorthQueryAspectTouch {
    certification_aspect_field_touch("title", "value")
}

fn certification_aspect_field_touch(
    aspect_label: &'static str,
    field_label: &'static str,
) -> WorthQueryAspectTouch {
    let aspect_key =
        AspectKey::new(aspect_label).expect("certification static aspect key should admit");
    let field_key =
        FieldKey::new(field_label).expect("certification static field key should admit");
    let field_path =
        CanonicalFieldPath::new([field_key]).expect("certification static field path should admit");
    WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path)
}

fn stable_certification_position(namespace: impl AsRef<str>, evidence: impl AsRef<str>) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.as_ref().bytes().chain(evidence.as_ref().bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211_u64);
    }
    acc
}

#[derive(Clone)]
pub(super) struct CertifiedAdmittedIntentFixture {
    pub(super) request: WorthQueryRawIntentAdmissionRequest,
    pub(super) decision: WorthQueryIntentAdmissionDecision,
    pub(super) plan: crate::intent_admission::WorthQueryAdmittedIntentPlan,
    pub(super) handoff: crate::intent_admission::WorthQueryAdmittedIntentExecutionHandoff,
    pub(super) binding: crate::intent_admission::WorthQueryAuthoritativeIntentExecutionBinding,
    pub(super) trace: crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope,
    pub(super) receipt: crate::runtime::WorthQueryIntentReceipt,
}

#[derive(Clone)]
pub(super) struct CertifiedAdvisoryIntentFixture {
    pub(super) request: WorthQueryRawIntentAdmissionRequest,
    pub(super) decision: WorthQueryIntentAdmissionDecision,
    pub(super) trace: crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope,
}

#[derive(Clone)]
pub(super) struct CertifiedViolationIntentFixture {
    pub(super) request: WorthQueryRawIntentAdmissionRequest,
    pub(super) decision: WorthQueryIntentAdmissionDecision,
    pub(super) trace: crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope,
}

#[derive(Clone)]
pub(super) struct CertifiedFailureIntentFixture {
    pub(super) request: WorthQueryRawIntentAdmissionRequest,
    pub(super) failure_digest: String,
    pub(super) execution_provenance_chain_digest: String,
    pub(super) trace: crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope,
}

pub(super) struct LegacyDelegationParityFixture {
    pub(super) authoritative_legacy: crate::runtime::WorthQueryIntentReceipt,
    pub(super) authoritative_canonical: crate::runtime::WorthQueryIntentReceipt,
    pub(super) effect_legacy: crate::runtime::WorthQueryEffectIntentReceipt,
    pub(super) effect_canonical: crate::runtime::WorthQueryEffectIntentReceipt,
    pub(super) read_current_legacy: crate::runtime::WorthQueryReadResult,
    pub(super) read_current_canonical: crate::runtime::WorthQueryReadResult,
    pub(super) read_basis_legacy: crate::runtime::WorthQueryReadResult,
    pub(super) read_basis_canonical: crate::runtime::WorthQueryReadResult,
}

pub(super) fn certified_admitted_intent_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CertifiedAdmittedIntentFixture {
    let declaration = authoritative_declaration("certification-admitted-intent");
    let request =
        WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(declaration.clone())
            .expect("authoritative request should build");
    let _eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let decision = admit_runtime_intent_request(request.clone());
    match &decision {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            let mut runtime = certification_runtime(execution);
            let plan = plan.clone();
            let handoff = plan.clone().into_execution_handoff();
            let authoritative_handoff = match handoff.clone() {
                Some(crate::intent_admission::WorthQueryAdmittedIntentExecutionHandoff::Authoritative(
                    handoff,
                )) => handoff,
                other => panic!("expected authoritative handoff, got {other:?}"),
            };
            let binding =
                runtime.prepare_authoritative_intent_execution_binding(authoritative_handoff);
            let receipt = runtime
                .execute_intent(execution, declaration)
                .expect("admitted certification intent should execute");
            let trace = receipt.decision_trace_envelope().clone();
            CertifiedAdmittedIntentFixture {
                request,
                decision,
                plan,
                handoff: handoff.expect("authoritative admitted plan should still mint a handoff"),
                binding,
                trace,
                receipt,
            }
        }
        other => panic!("expected admitted decision, got {other:?}"),
    }
}

pub(super) fn certified_advisory_intent_fixture() -> CertifiedAdvisoryIntentFixture {
    let declaration = authoritative_declaration("certification-advisory-intent");
    let request =
        WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(declaration)
            .expect("authoritative request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let advisory = WorthQueryIntentAdvisoryDecision::new(
        request.family(),
        request.entrypoint(),
        "materialized-detail-advisory",
        "full execution is intentionally deferred",
        request.request_digest(),
        eligibility.eligibility_digest(),
    );
    let decision = WorthQueryIntentAdmissionDecision::Advisory(advisory);
    let review = WorthQueryRuntimeIntentAdmissionReviewData::from_decision(
        request.clone(),
        decision.clone(),
    );
    let trace = review
        .decision_trace_envelope()
        .expect("advisory review should preserve a trace")
        .clone();
    CertifiedAdvisoryIntentFixture {
        request,
        decision,
        trace,
    }
}

pub(super) fn certified_violation_intent_fixture() -> CertifiedViolationIntentFixture {
    let declaration = authoritative_declaration("certification-violation-intent");
    let request = WorthQueryRawIntentAdmissionRequest::effect_runtime_entrypoint(declaration)
        .expect("violation request should still build");
    let _eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let decision = admit_runtime_intent_request(request.clone());
    let review = WorthQueryRuntimeIntentAdmissionReviewData::from_decision(
        request.clone(),
        decision.clone(),
    );
    let trace = review
        .decision_trace_envelope()
        .expect("violation review should preserve a trace")
        .clone();
    CertifiedViolationIntentFixture {
        request,
        decision,
        trace,
    }
}

pub(super) fn certified_failure_intent_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CertifiedFailureIntentFixture {
    let declaration = authoritative_declaration("certification-failure-intent");
    let request =
        WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(declaration.clone())
            .expect("authoritative request should build");
    let mut runtime = certification_runtime_with_invariant_violation_authority(execution);
    let error = runtime
        .execute_intent(execution, declaration)
        .expect_err("invariant violation should preserve denial evidence");
    match error {
        crate::runtime::WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } => {
            let trace = evidence
                .decision_trace_envelope()
                .expect("execution-time denial should retain a trace")
                .clone();
            CertifiedFailureIntentFixture {
                request,
                failure_digest: evidence
                    .denial_digest()
                    .terminal_projection_for_reporting()
                    .to_string(),
                execution_provenance_chain_digest: evidence
                    .execution_provenance()
                    .expect("execution-time denial should retain provenance")
                    .execution_provenance_chain_digest()
                    .to_string(),
                trace,
            }
        }
        other => panic!("expected execution-time denial evidence, got {other:?}"),
    }
}

pub(super) fn legacy_delegation_parity_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> LegacyDelegationParityFixture {
    let read_fixture = read_delegation_parity_fixture(execution);
    let declaration = authoritative_declaration("certification-authoritative-parity-intent");
    let mut delegated_runtime = certification_runtime(execution);
    let authoritative_legacy = delegated_runtime
        .execute_intent(execution, declaration.clone())
        .expect("legacy authoritative entrypoint should execute");

    let mut canonical_runtime = certification_runtime(execution);
    let canonical_handoff = canonical_runtime
        .admit_authoritative_intent_for_execution(declaration.clone())
        .expect("canonical authoritative handoff should admit");
    let canonical_binding =
        canonical_runtime.prepare_authoritative_intent_execution_binding(canonical_handoff);
    let authoritative_canonical = canonical_runtime
        .execute_authoritative_intent_execution_binding(execution, canonical_binding)
        .expect("canonical authoritative execution should succeed");

    let mut delegated_effect_runtime = certification_runtime(execution);
    let delegated_live = delegated_effect_runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            execution,
            "certification.effect-live",
            certification_task_live_request(),
            certification_task_schema(),
        )
        .expect("live view should declare");
    let delegated_effect = delegated_effect_runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.certification.reconcile",
            WorthQueryEffectTrigger::live_view(&delegated_live, [title_value_touch()]),
            "strategy.intent.reconcile",
        ))
        .expect("effect should declare");
    delegated_effect_runtime
        .write(
            execution,
            WorthQueryWriteCommand::UpdateAspect {
                entity_identity: certification_entity_identity("task-1"),
                aspect: crate::facade::runtime::WorthQueryAuthoredAspectMutation::new_set(
                    title_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "title from delegated effect",
                    ),
                )
                .expect("delegated effect aspect should admit"),
            },
        )
        .expect("delegated effect write should queue");
    let effect_legacy = delegated_effect_runtime
        .execute_next_effect_write_intent(
            execution,
            &delegated_effect,
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("legacy effect entrypoint should execute");

    let mut canonical_effect_runtime = certification_runtime(execution);
    let canonical_live = canonical_effect_runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            execution,
            "certification.effect-live",
            certification_task_live_request(),
            certification_task_schema(),
        )
        .expect("canonical live view should declare");
    let canonical_effect = canonical_effect_runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.certification.reconcile",
            WorthQueryEffectTrigger::live_view(&canonical_live, [title_value_touch()]),
            "strategy.intent.reconcile",
        ))
        .expect("canonical effect should declare");
    canonical_effect_runtime
        .write(
            execution,
            WorthQueryWriteCommand::UpdateAspect {
                entity_identity: certification_entity_identity("task-1"),
                aspect: crate::facade::runtime::WorthQueryAuthoredAspectMutation::new_set(
                    title_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "title from delegated effect",
                    ),
                )
                .expect("canonical effect aspect should admit"),
            },
        )
        .expect("canonical effect write should queue");
    let (pending_delivery, canonical_handoff) = canonical_effect_runtime
        .admit_next_effect_write_intent_for_execution(
            canonical_effect.name(),
            "1.0",
            "effect.intent.input.v1",
        )
        .expect("canonical effect handoff should admit");
    let canonical_binding = canonical_effect_runtime
        .prepare_effect_intent_execution_binding(canonical_handoff, &pending_delivery);
    let effect_canonical = canonical_effect_runtime
        .execute_effect_intent_execution_binding(execution, canonical_binding)
        .expect("canonical effect execution should succeed");

    LegacyDelegationParityFixture {
        authoritative_legacy,
        authoritative_canonical,
        effect_legacy,
        effect_canonical,
        read_current_legacy: read_fixture.current_legacy,
        read_current_canonical: read_fixture.current_canonical,
        read_basis_legacy: read_fixture.basis_legacy,
        read_basis_canonical: read_fixture.basis_canonical,
    }
}

mod authoritative_declaration;
pub(super) use authoritative_declaration::authoritative_declaration;
