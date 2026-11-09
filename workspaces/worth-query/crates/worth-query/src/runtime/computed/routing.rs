use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
pub(super) mod route_result;

use super::*;
use crate::memory_workspace::WorthQueryMutationKind;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct WorthQueryComputedRouteResult {
    affected_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
    refresh_fallback: bool,
    considered_view_count: usize,
}

fn retained_upstream_inputs(
    declaration: &WorthQueryDerivedView,
    retained_live_rows: &BTreeMap<
        WorthQueryLiveArtifactTarget,
        Vec<crate::memory_workspace::WorthQueryEntity>,
    >,
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
) -> WorthQueryRetainedUpstreamInputs {
    WorthQueryRetainedUpstreamInputs::from_retained_computed_rows(
        declaration.upstream_live_views().iter().filter_map(|name| {
            let target = WorthQueryLiveArtifactTarget::from_view_name(name);
            retained_live_rows
                .get(&target)
                .cloned()
                .map(|rows| (target, rows))
        }),
        declaration
            .upstream_derived_views()
            .iter()
            .filter_map(|name| {
                let target = WorthQueryDerivedMaterializationTarget::new(name);
                derived_views
                    .get(&target)
                    .map(|view| (target, view.materialization.retained_rows().to_vec()))
            }),
    )
}

fn reaches_derived_view(
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    start: &WorthQueryDerivedMaterializationTarget,
    target: &WorthQueryDerivedMaterializationTarget,
    seen: &mut BTreeSet<WorthQueryDerivedMaterializationTarget>,
) -> bool {
    if start == target {
        return true;
    }
    if !seen.insert(start.clone()) {
        return false;
    }
    let Some(view) = derived_views.get(start) else {
        return false;
    };
    view.declaration
        .upstream_derived_views()
        .iter()
        .any(|upstream| {
            reaches_derived_view(
                derived_views,
                &WorthQueryDerivedMaterializationTarget::new(upstream),
                target,
                seen,
            )
        })
}

fn topological_derived_order(
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
) -> Vec<WorthQueryDerivedMaterializationTarget> {
    let mut ordered = Vec::new();
    let mut permanent = BTreeSet::new();
    let mut temporary = BTreeSet::new();
    for target in derived_views.keys() {
        visit_derived_view(
            target,
            derived_views,
            &mut permanent,
            &mut temporary,
            &mut ordered,
        );
    }
    ordered
}

fn visit_derived_view(
    target: &WorthQueryDerivedMaterializationTarget,
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    permanent: &mut BTreeSet<WorthQueryDerivedMaterializationTarget>,
    temporary: &mut BTreeSet<WorthQueryDerivedMaterializationTarget>,
    ordered: &mut Vec<WorthQueryDerivedMaterializationTarget>,
) {
    if permanent.contains(target) || !temporary.insert(target.clone()) {
        return;
    }
    if let Some(view) = derived_views.get(target) {
        for upstream in view.declaration.upstream_derived_views() {
            visit_derived_view(
                &WorthQueryDerivedMaterializationTarget::new(upstream),
                derived_views,
                permanent,
                temporary,
                ordered,
            );
        }
    }
    temporary.remove(target);
    permanent.insert(target.clone());
    ordered.push(target.clone());
}

fn relevant_source_deltas(
    view: &WorthQueryDerivedViewRuntime,
    receipt: &WorthQueryMutationReceipt,
    emitted_by_view: &BTreeMap<WorthQueryDerivedMaterializationTarget, Vec<WorthQueryDerivedPatch>>,
) -> Result<Vec<WorthQueryMutationDelta>, IdentityDerivationDenial> {
    let mut deltas = Vec::new();
    let accepts_authoritative_deltas = view.declaration.upstream_derived_views().is_empty();
    if accepts_authoritative_deltas {
        deltas.extend(
            receipt
                .deltas
                .iter()
                .filter(|delta| mutation_delta_matches_view(view, delta))
                .cloned(),
        );
    }

    for upstream in view.declaration.upstream_derived_views() {
        let upstream_target = WorthQueryDerivedMaterializationTarget::new(upstream);
        let Some(upstream_patches) = emitted_by_view.get(&upstream_target) else {
            continue;
        };
        for patch in upstream_patches {
            let delta = patch.to_mutation_delta(upstream)?;
            if mutation_delta_matches_view(view, &delta) {
                deltas.push(delta);
            }
        }
    }

    Ok(deltas)
}

fn mutation_delta_matches_view(
    view: &WorthQueryDerivedViewRuntime,
    delta: &WorthQueryMutationDelta,
) -> bool {
    if matches!(
        delta.kind(),
        WorthQueryMutationKind::Created | WorthQueryMutationKind::Deleted
    ) {
        return true;
    }
    delta.admitted_touched_aspects().is_empty()
        || delta.admitted_touched_aspects().iter().any(|aspect_touch| {
            view.declaration
                .dependency_aspect_touches()
                .iter()
                .any(|dependency| dependency.matches_or_contains(aspect_touch))
        })
}

fn effective_patch_aspects(
    view: &WorthQueryDerivedView,
    delta: &WorthQueryMutationDelta,
) -> Vec<WorthQueryAspectTouch> {
    if !view.produced_aspect_touches().is_empty() {
        view.produced_aspect_touches().to_vec()
    } else {
        delta.admitted_touched_aspects().to_vec()
    }
}
