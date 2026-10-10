use crate::facade::runtime::{
    WorthQueryAuthorityLane, WorthQueryBranchOptions, WorthQueryEffectHandle,
    WorthQueryIntentDeclaration, WorthQueryIntentInput, WorthQueryLiveView,
    WorthQueryPreviewOptions, WorthQueryRuntimeFacadeFamily, WorthQueryRuntimePublicApiContract,
    WorthQueryWorkspace,
};
use crate::identity::hash_parts;
use crate::runtime::WorthQueryUnrefinedLiveShape;
use crate::WorthQuerySessionLabel;

use super::transcript_aspect_touch;
use super::transcripts::TranscriptSpec;

pub(super) fn preview_proof(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    live: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    computed: &crate::facade::runtime::WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
    effect: &WorthQueryEffectHandle<WorthQueryUnrefinedLiveShape>,
    spec: &TranscriptSpec,
) -> (String, usize) {
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            WorthQuerySessionLabel::scoped_strs(
                "runtime-api-stabilization",
                [format!("{}.preview", spec.family)],
            )
            .expect("preview label should build"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("transcript preview should open");
    let live_binding = preview.use_view(live);
    let computed_binding = preview.use_computed(computed);
    let effect_binding = preview
        .use_effect(request_execution, effect)
        .expect("effect should bind in preview");
    let preview_write = preview
        .insert(request_execution, spec.collection, |entity| {
            let entity = entity.set_aspect(
                transcript_aspect_touch("identity.id"),
                string_aspect_value(format!("{}-preview-1", spec.family)),
            );
            spec.produced_aspects
                .iter()
                .enumerate()
                .fold(entity, |builder, (index, aspect)| {
                    builder.set_aspect(
                        transcript_aspect_touch(*aspect),
                        string_aspect_value(format!("{}-preview-{index}", spec.family)),
                    )
                })
        })
        .expect("preview write should stage");
    let preview_intent = preview
        .execute_intent(intent_declaration(
            format!("{}.preview-intent", spec.intent_name),
            spec.collection,
        ))
        .expect("preview intent should stage");
    let outcome = preview.discard();
    assert_eq!(
        preview_write.authority_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(
        preview_intent.target_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(
        outcome.authoritative_residue_count(),
        0,
        "preview transcript must not leak authoritative writes"
    );
    assert!(
        outcome.preview_binding_count() >= 3,
        "preview should bind live, computed, and effect handles"
    );
    (
        hash_parts(&[
            binding_digest(&live_binding),
            binding_digest(&computed_binding),
            binding_digest(&effect_binding),
            preview_intent.receipt_digest().to_string(),
            outcome.closeout_evidence().closeout_digest().to_string(),
        ]),
        outcome.write_count() + outcome.pending_write_intent_residue_count(),
    )
}

pub(super) fn branch_proof(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    spec: &TranscriptSpec,
) -> String {
    let mut branch = workspace
        .branch_with_options(
            request_execution,
            WorthQuerySessionLabel::scoped_strs(
                "runtime-api-stabilization",
                [format!("{}.branch", spec.family)],
            )
            .expect("branch label should build"),
            WorthQueryBranchOptions::sandboxed_write_intent(),
        )
        .expect("transcript branch should open");
    let receipt = branch
        .execute_intent(intent_declaration(
            format!("{}.branch-intent", spec.intent_name),
            spec.collection,
        ))
        .expect("branch intent should stay branch-local");
    assert_eq!(
        receipt.target_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
    assert_eq!(branch.branch_intent_receipts().len(), 1);
    receipt.receipt_digest().to_string()
}

pub(super) fn support_gated_neighbor_denials(
    contract: &WorthQueryRuntimePublicApiContract,
    families: &[WorthQueryRuntimeFacadeFamily],
) -> Vec<String> {
    families
        .iter()
        .map(|family| {
            let row = contract
                .family(*family)
                .expect("future neighbor family should have contract row");
            assert!(
                row.status()
                    != crate::facade::runtime::WorthQueryRuntimeFamilySupportStatus::Supported,
                "future neighbor must not be silently admitted in public stabilization transcripts"
            );
            hash_parts(&[
                format!("family:{}", family.as_str()),
                format!("status:{}", row.status().as_str()),
                format!("reason:{}", row.reason().unwrap_or("none")),
                format!("contract:{}", row.contract_digest()),
            ])
        })
        .collect()
}

fn binding_digest(
    binding: &crate::facade::runtime::WorthQueryPreviewHandleBindingEvidence,
) -> String {
    hash_parts(&[
        format!("label_identity:{}", binding.label_identity().as_str()),
        format!("handle:{}", binding.handle_name()),
        format!("family:{}", binding.family().as_str()),
        format!("source:{}", binding.source_lane().as_str()),
        format!("preview:{}", binding.preview_lane().as_str()),
        format!("policy:{}", binding.effect_policy().as_str()),
        format!(
            "disposition:{}",
            binding
                .effect_disposition()
                .map(|disposition| disposition.as_str())
                .unwrap_or("none")
        ),
    ])
}

pub(super) fn intent_declaration(
    name: impl Into<String>,
    collection: &str,
) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        "strategy.intent.transcript",
        "1.0",
        "transcript.intent.input.v1",
        WorthQueryIntentInput::object([
            ("collection", WorthQueryIntentInput::string(collection)),
            (
                "entity",
                WorthQueryIntentInput::string("transcript-entity-1"),
            ),
        ]),
    )
}

fn string_aspect_value(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}
