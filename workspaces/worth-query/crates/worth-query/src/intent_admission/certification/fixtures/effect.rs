use super::certification_entity_identity;
use super::runtime::{
    certification_runtime, certification_task_live_request, certification_task_schema,
};
use super::title_value_touch;
use crate::facade::runtime::{WorthQueryEffectDeclaration, WorthQueryEffectTrigger};
use crate::runtime::WorthQueryUnrefinedLiveShape;

#[derive(Clone)]
pub(in crate::intent_admission::certification) struct CertifiedEffectIntentFixture {
    pub(in crate::intent_admission::certification) request_digest: String,
    pub(in crate::intent_admission::certification) eligibility_digest: String,
    pub(in crate::intent_admission::certification) decision_digest: String,
    pub(in crate::intent_admission::certification) handoff_digest: String,
    pub(in crate::intent_admission::certification) binding_digest: String,
    pub(in crate::intent_admission::certification) trace_digest: String,
    pub(in crate::intent_admission::certification) receipt_digest: String,
    pub(in crate::intent_admission::certification) execution_provenance_chain_digest: String,
}

pub(in crate::intent_admission::certification) fn certified_effect_intent_fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CertifiedEffectIntentFixture {
    let mut runtime = certification_runtime(execution);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            execution,
            "certification.effect-live",
            certification_task_live_request(),
            certification_task_schema(),
        )
        .expect("effect certification live view should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.certification.reconcile",
            WorthQueryEffectTrigger::live_view(&live, [title_value_touch()]),
            "strategy.intent.reconcile",
        ))
        .expect("effect certification effect should declare");
    runtime
        .write(
            execution,
            crate::facade::runtime::WorthQueryWriteCommand::UpdateAspect {
                entity_identity: certification_entity_identity("task-1"),
                aspect: crate::facade::runtime::WorthQueryAuthoredAspectMutation::new_set(
                    title_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "title from certification effect",
                    ),
                )
                .expect("effect certification aspect should admit"),
            },
        )
        .expect("effect certification write should queue");

    let review = runtime
        .next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .review()
        .expect("effect certification review should succeed");
    let request_digest = review.request().request_digest().to_string();
    let eligibility_digest = review.eligibility().eligibility_digest().to_string();
    let decision_digest = match review.decision() {
        crate::facade::runtime::WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            plan.decision_digest().to_string()
        }
        other => panic!("effect certification review should admit, got {other:?}"),
    };
    let admitted = review
        .admit()
        .expect("effect certification admitted path should resolve");
    let handoff_digest = admitted.handoff().handoff_digest().to_string();
    let binding_digest = admitted.execution_binding().binding_digest().to_string();
    let receipt = admitted
        .execute(execution)
        .expect("effect certification admitted execution should succeed");
    let trace_digest = receipt.decision_trace_envelope().trace_digest().to_string();
    let execution_provenance_chain_digest = receipt.execution_provenance_chain_digest().to_string();
    let receipt_digest = receipt.intent_receipt().receipt_digest().to_string();

    CertifiedEffectIntentFixture {
        request_digest,
        eligibility_digest,
        decision_digest,
        handoff_digest,
        binding_digest,
        trace_digest,
        receipt_digest,
        execution_provenance_chain_digest,
    }
}
