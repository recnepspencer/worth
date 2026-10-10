use crate::facade::foundation::AspectFieldKey;
use crate::facade::runtime::{
    WorthQueryInspection, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimePublicApiTranscriptEvidence,
};
use crate::identity::hash_parts;
use crate::runtime::WorthQueryUnrefinedLiveShape;

use super::transcript_aspect_touch;
use super::transcript_maintainer::TranscriptMaintainer;
use super::transcript_runtime::transcript_runtime;
use super::transcript_session_proofs::{
    branch_proof, intent_declaration, preview_proof, support_gated_neighbor_denials,
};

pub(super) fn workflow_editor_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    execute_transcript(
        request_execution,
        TranscriptSpec {
            family: "workflow-editor",
            collection: "WorkflowNode",
            view_name: "workflow.editor.canvas",
            first_computed: "workflow.editor.section-readiness",
            second_computed: "workflow.editor.workflow-readiness",
            effect_name: "workflow.editor.publish-readiness",
            intent_name: "workflow.editor.commit-intent",
            produced_aspects: &["validation.state", "layout.frame", "runtimeValue.preview"],
            neighbor_families: &[WorthQueryRuntimeFacadeFamily::StoreBackedExecution],
            assertion_floor: 12,
        },
    )
}

pub(super) fn geometry_kernel_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    execute_transcript(
        request_execution,
        TranscriptSpec {
            family: "geometry-kernel",
            collection: "ModelEntity",
            view_name: "geometry.kernel.topology",
            first_computed: "geometry.kernel.nurbs-evaluation",
            second_computed: "geometry.kernel.fillet-solver-readiness",
            effect_name: "geometry.kernel.persist-solver-output",
            intent_name: "geometry.kernel.commit-solve",
            produced_aspects: &["topology.edge", "surface.trim", "solver.residue"],
            neighbor_families: &[WorthQueryRuntimeFacadeFamily::DurableArtifacts],
            assertion_floor: 12,
        },
    )
}

pub(super) fn table_spreadsheet_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    execute_transcript(
        request_execution,
        TranscriptSpec {
            family: "table-spreadsheet",
            collection: "SheetCell",
            view_name: "table.sheet.visible-window",
            first_computed: "table.sheet.formula-values",
            second_computed: "table.sheet.dropdown-domain",
            effect_name: "table.sheet.persist-batched-edit",
            intent_name: "table.sheet.commit-edit",
            produced_aspects: &["formula.value", "dropdown.domain", "layout.width"],
            neighbor_families: &[WorthQueryRuntimeFacadeFamily::StoreBackedExecution],
            assertion_floor: 12,
        },
    )
}

pub(super) fn composed_runtime_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    execute_transcript(
        request_execution,
        TranscriptSpec {
            family: "composed-runtime",
            collection: "ComposedNode",
            view_name: "composed.runtime.surface",
            first_computed: "composed.runtime.derived-a",
            second_computed: "composed.runtime.derived-b",
            effect_name: "composed.runtime.pending-intent-effect",
            intent_name: "composed.runtime.commit-intent",
            produced_aspects: &[
                "invariant.state",
                "branch.preview",
                "expression.output",
                "intent.commit",
            ],
            neighbor_families: &[
                WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
                WorthQueryRuntimeFacadeFamily::DurableArtifacts,
            ],
            assertion_floor: 16,
        },
    )
}

pub(super) fn composed_runtime_hostile_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    execute_transcript(
        request_execution,
        TranscriptSpec {
            family: "composed-runtime-hostile",
            collection: "ComposedNode",
            view_name: "composed.runtime.surface.hostile",
            first_computed: "composed.runtime.derived-a.hostile",
            second_computed: "composed.runtime.derived-b.hostile",
            effect_name: "composed.runtime.pending-intent-effect.hostile",
            intent_name: "composed.runtime.commit-intent.hostile",
            produced_aspects: &[
                "invariant.state",
                "branch.preview",
                "expression.output",
                "intent.commit",
                "store.debt-denial",
                "durable.debt-denial",
            ],
            neighbor_families: &[
                WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
                WorthQueryRuntimeFacadeFamily::DurableArtifacts,
            ],
            assertion_floor: 18,
        },
    )
}

pub(super) struct TranscriptSpec {
    pub(super) family: &'static str,
    pub(super) collection: &'static str,
    view_name: &'static str,
    first_computed: &'static str,
    second_computed: &'static str,
    effect_name: &'static str,
    pub(super) intent_name: &'static str,
    pub(super) produced_aspects: &'static [&'static str],
    neighbor_families: &'static [WorthQueryRuntimeFacadeFamily],
    assertion_floor: usize,
}

fn execute_transcript(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    spec: TranscriptSpec,
) -> WorthQueryRuntimePublicApiTranscriptEvidence {
    let mut workspace = transcript_runtime(request_execution, spec.produced_aspects)
        .workspace(format!("{}.workspace", spec.family))
        .expect("transcript runtime should expose a named workspace");
    let live = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(request_execution, spec.view_name, |q| {
            q.from(spec.collection)
                .select(
                    std::iter::once(live_field("identity.id")).chain(
                        spec.produced_aspects
                            .iter()
                            .map(|aspect| live_field(aspect)),
                    ),
                )
                .order_by(live_field("identity.id"))
                .schema_basis(format!("runtime-transcript-{}", spec.family))
                .as_surface(spec.view_name)
        })
        .expect("transcript live surface should declare");
    let first = workspace
        .computed::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            spec.first_computed,
            |c| {
                c.depends_on_live(&live)
                    .reads(
                        spec.produced_aspects
                            .iter()
                            .map(|aspect| transcript_aspect_touch(*aspect)),
                    )
                    .produces(
                        spec.produced_aspects
                            .iter()
                            .map(|aspect| transcript_aspect_touch(format!("{aspect}.derived"))),
                    )
            },
            TranscriptMaintainer {
                prefix: spec.family,
                replace: false,
            },
        )
        .expect("first computed transcript surface should declare");
    let second = workspace
        .computed::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            spec.second_computed,
            |c| {
                c.depends_on_computed(&first)
                    .reads(
                        spec.produced_aspects
                            .iter()
                            .map(|aspect| transcript_aspect_touch(format!("{aspect}.derived"))),
                    )
                    .produces(
                        spec.produced_aspects
                            .iter()
                            .map(|aspect| transcript_aspect_touch(format!("{aspect}.ready"))),
                    )
            },
            TranscriptMaintainer {
                prefix: spec.family,
                replace: true,
            },
        )
        .expect("nested computed transcript surface should declare");
    let effect = workspace
        .effect::<WorthQueryUnrefinedLiveShape>(spec.effect_name, |e| {
            e.when_computed(
                &second,
                spec.produced_aspects
                    .iter()
                    .map(|aspect| transcript_aspect_touch(format!("{aspect}.ready"))),
            )
            .condition_expression(
                format!("{}.custom-expression", spec.family),
                spec.produced_aspects
                    .iter()
                    .map(|aspect| transcript_aspect_touch(format!("{aspect}.ready"))),
                [transcript_aspect_touch(format!(
                    "{}.effect-output",
                    spec.family
                ))],
            )
            .write_intent("strategy.intent.transcript")
            .meaningful_change_suppression()
        })
        .expect("write-intent effect transcript surface should declare");

    let (preview_digest, preview_residue_count) = preview_proof(
        request_execution,
        &mut workspace,
        &live,
        &second,
        &effect,
        &spec,
    );
    let intent = workspace
        .intent(
            request_execution,
            intent_declaration(spec.intent_name, spec.collection),
        )
        .expect("authoritative transcript intent should commit");
    let write = workspace
        .insert(request_execution, spec.collection, |entity| {
            let entity = entity.set_aspect(
                transcript_aspect_touch("identity.id"),
                string_aspect_value(format!("{}-entity-1", spec.family)),
            );
            spec.produced_aspects
                .iter()
                .enumerate()
                .fold(entity, |builder, (index, aspect)| {
                    builder.set_aspect(
                        transcript_aspect_touch(*aspect),
                        string_aspect_value(format!("{}-value-{index}", spec.family)),
                    )
                })
        })
        .expect("transcript write should route live, computed, and effect artifacts");
    assert!(
        write.pending_write_intent_count() >= 1,
        "transcript effect should leave a pending write intent"
    );
    assert!(
        write
            .terminal_affected_live_view_ids_projection()
            .iter()
            .any(|view| view == spec.view_name),
        "transcript write should affect its durable live view"
    );
    let patches = workspace.observe(&live);
    assert!(
        !patches.query_delivery_batches.is_empty(),
        "transcript live surface should emit query delivery batches"
    );
    let live_inspection = match workspace
        .inspect(request_execution, &live)
        .expect("live transcript surface should inspect")
    {
        WorthQueryInspection::LiveView(inspection) => inspection,
        other => panic!("expected live view inspection, got {other:?}"),
    };
    let live_installation_digest = live_inspection
        .installation_projection()
        .label()
        .as_str()
        .to_string();
    let live_active_lane_digest = live_inspection
        .active_lane_projection()
        .label()
        .as_str()
        .to_string();
    let computed_inspection = match workspace
        .inspect(request_execution, &second)
        .expect("nested computed transcript surface should inspect")
    {
        WorthQueryInspection::DerivedView(inspection) => inspection,
        other => panic!("expected derived view inspection, got {other:?}"),
    };
    assert!(
        computed_inspection.pending_incremental_patch_count() >= 1,
        "nested computed should retain incremental patch evidence"
    );
    let effect_inspection = match workspace
        .inspect(request_execution, &effect)
        .expect("effect transcript surface should inspect")
    {
        WorthQueryInspection::Effect(inspection) => inspection,
        other => panic!("expected effect inspection, got {other:?}"),
    };
    assert!(
        effect_inspection.pending_write_intent_count() >= 1,
        "effect inspection should expose pending intent residue"
    );
    let effect_intent = workspace
        .next_effect_intent(
            request_execution,
            &effect,
            "1.0",
            "transcript.effect.intent.v1",
        )
        .expect("effect pending intent should execute through intent authority");
    let effect_intent_inspection = match workspace
        .inspect(request_execution, &effect_intent)
        .expect("effect intent receipt should inspect")
    {
        WorthQueryInspection::EffectIntentReceipt(inspection) => inspection,
        other => panic!("expected effect intent receipt inspection, got {other:?}"),
    };
    let branch_digest = branch_proof(request_execution, &mut workspace, &spec);
    let intent_inspection = match workspace
        .inspect(request_execution, &intent)
        .expect("authoritative intent receipt should inspect")
    {
        WorthQueryInspection::IntentReceipt(inspection) => inspection,
        other => panic!("expected intent receipt inspection, got {other:?}"),
    };
    let support_contract = workspace.public_api_contract();
    let denial_digests = support_gated_neighbor_denials(&support_contract, spec.neighbor_families);
    let state = workspace
        .state(request_execution, &second)
        .expect("golden transcript should use the public state boundary");
    let authority_lane_digest = hash_parts(&[
        format!("live:{live_active_lane_digest}"),
        format!("computed:{}", computed_inspection.produced_aspect_digest()),
        format!("effect:{}", effect_inspection.trigger_digest()),
        format!("preview:{preview_digest}"),
        format!("branch:{branch_digest}"),
        format!(
            "effect_intent:{}",
            effect_intent_inspection.inspection_digest()
        ),
    ]);
    WorthQueryRuntimePublicApiTranscriptEvidence::new(
        spec.family,
        support_contract.contract_digest(),
        state.state_digest().terminal_projection_for_reporting(),
        live_installation_digest,
        computed_inspection.inspection_digest(),
        effect_inspection.inspection_digest(),
        intent.receipt_digest(),
        hash_parts(&[
            intent_inspection.inspection_digest().to_string(),
            effect_intent_inspection.inspection_digest().to_string(),
            branch_digest,
        ]),
        denial_digests,
        preview_residue_count + write.pending_write_intent_count(),
        authority_lane_digest,
        spec.assertion_floor,
    )
}

fn live_field(authored_field_text: &str) -> AspectFieldKey {
    let (aspect, field) = authored_field_text
        .split_once('.')
        .expect("transcript authored field should use aspect.field form");
    AspectFieldKey::from_authoring_parts(aspect, field)
        .expect("transcript aspect field should be valid")
}

fn string_aspect_value(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}
