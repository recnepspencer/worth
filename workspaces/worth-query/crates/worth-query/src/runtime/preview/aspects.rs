use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget};

pub(super) fn relevant_live_aspects(
    request: &DeclarativeLiveQueryRequest,
    deltas: &[WorthQueryMutationDelta],
) -> Result<Vec<WorthQueryAspectTouch>, IdentityDerivationDenial> {
    let mut aspects: BTreeSet<WorthQueryAspectTouch> = BTreeSet::new();
    let request_target = request.target_collection_identity()?;
    for delta in deltas {
        if !delta
            .target_collection_identity()
            .same_target_collection_as(&request_target)
        {
            continue;
        }
        if delta.admitted_touched_aspects().is_empty()
            || matches!(
                delta.kind,
                WorthQueryMutationKind::Created | WorthQueryMutationKind::Deleted
            )
        {
            for field in request.projection() {
                aspects.insert(preview_field_touch(field.source_field_key()));
            }
            continue;
        }
        for touch in delta.admitted_touched_aspects() {
            if request.projection().iter().any(|field| {
                let projected = preview_field_touch(field.source_field_key());
                touch.matches_or_contains(&projected)
            }) {
                aspects.insert(touch.clone());
            }
        }
    }
    Ok(aspects.into_iter().collect())
}

pub(super) fn relevant_computed_aspects(
    runtime: &WorthQueryDerivedViewRuntime,
    live_affected: &BTreeMap<WorthQueryLiveArtifactTarget, Vec<WorthQueryAspectTouch>>,
    computed_affected: &BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        Vec<WorthQueryAspectTouch>,
    >,
) -> Vec<WorthQueryAspectTouch> {
    let mut matched = BTreeSet::new();
    for upstream in runtime.declaration.upstream_live_views() {
        let upstream = WorthQueryLiveArtifactTarget::from_view_name(upstream.to_string());
        if let Some(aspects) = live_affected.get(&upstream) {
            matched.extend(aspects.iter().cloned());
        }
    }
    for upstream in runtime.declaration.upstream_derived_views() {
        let upstream = WorthQueryDerivedMaterializationTarget::new(upstream.to_string());
        if let Some(aspects) = computed_affected.get(&upstream) {
            matched.extend(aspects.iter().cloned());
        }
    }
    if matched.is_empty() {
        return Vec::new();
    }
    if !runtime.declaration.produced_aspect_touches().is_empty() {
        runtime.declaration.produced_aspect_touches().to_vec()
    } else {
        matched.into_iter().collect()
    }
}

pub(super) fn relevant_effect_aspects(
    inspected: &WorthQueryEffectInspectionEvidence,
    live_affected: &BTreeMap<WorthQueryLiveArtifactTarget, Vec<WorthQueryAspectTouch>>,
    computed_affected: &BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        Vec<WorthQueryAspectTouch>,
    >,
) -> Vec<WorthQueryAspectTouch> {
    let source_aspects = match inspected.trigger_source_kind() {
        WorthQueryEffectTriggerSourceKind::LiveView => live_affected.get(
            &WorthQueryLiveArtifactTarget::from_view_name(inspected.trigger_source().to_string()),
        ),
        WorthQueryEffectTriggerSourceKind::ComputedView => computed_affected.get(
            &WorthQueryDerivedMaterializationTarget::new(inspected.trigger_source().to_string()),
        ),
    };
    let Some(source_aspects) = source_aspects else {
        return Vec::new();
    };
    source_aspects
        .iter()
        .filter(|aspect| {
            inspected
                .trigger_aspect_touches()
                .iter()
                .any(|declared| declared.matches_or_contains(aspect))
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn preview_field_touch(field: &crate::authoring::AspectFieldKey) -> WorthQueryAspectTouch {
    WorthQueryAspectTouch::from_native_parts(
        field.native_aspect_key(),
        Some(worth_foundational::facade::CanonicalFieldPath::single(
            field.native_field_key(),
        )),
    )
}
