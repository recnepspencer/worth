use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl WorthQueryComputedRouteResult {
    pub(in crate::runtime) fn new(
        affected_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
        refresh_fallback: bool,
        considered_view_count: usize,
    ) -> Self {
        Self {
            affected_view_targets,
            refresh_fallback,
            considered_view_count,
        }
    }

    pub(in crate::runtime) fn affected_view_targets(
        self,
    ) -> Vec<WorthQueryDerivedMaterializationTarget> {
        self.affected_view_targets
    }

    pub(in crate::runtime) fn refresh_fallback(&self) -> bool {
        self.refresh_fallback
    }

    pub(in crate::runtime) fn considered_view_count(&self) -> usize {
        self.considered_view_count
    }
}

pub(in crate::runtime) fn admit_derived_view_declaration(
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    live_view_targets: &BTreeSet<WorthQueryLiveArtifactTarget>,
    declaration: &WorthQueryDerivedView,
) -> Result<(), WorthQueryComputedAdmissionError> {
    let name = declaration.name();
    for upstream in declaration.upstream_live_views() {
        if !live_view_targets.contains(&WorthQueryLiveArtifactTarget::from_view_name(upstream)) {
            return Err(WorthQueryComputedAdmissionError::MissingUpstreamLive {
                upstream: upstream.clone(),
            });
        }
    }
    for upstream in declaration.upstream_derived_views() {
        if upstream == name {
            return Err(WorthQueryComputedAdmissionError::SelfDependency);
        }
        let upstream_target = WorthQueryDerivedMaterializationTarget::new(upstream);
        if !derived_views.contains_key(&upstream_target) {
            return Err(WorthQueryComputedAdmissionError::MissingUpstreamComputed {
                upstream: upstream.clone(),
            });
        }
        if reaches_derived_view(
            derived_views,
            &upstream_target,
            &WorthQueryDerivedMaterializationTarget::new(name),
            &mut BTreeSet::new(),
        ) {
            return Err(WorthQueryComputedAdmissionError::Cycle {
                upstream: upstream.clone(),
            });
        }
    }
    Ok(())
}

pub(in crate::runtime) fn insert_derived_runtime(
    derived_views: &mut BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        WorthQueryDerivedViewRuntime,
    >,
    dependency_index: &mut WorthQueryComputedDependencyIndex,
    view: WorthQueryDerivedView,
    maintainer: Option<Box<dyn WorthQueryDerivedViewMaintainer>>,
) {
    dependency_index.register(&view);
    let target = WorthQueryDerivedMaterializationTarget::new(view.name());
    derived_views.insert(target, WorthQueryDerivedViewRuntime::new(view, maintainer));
}

pub(in crate::runtime) fn route_derived_view_patches(
    derived_views: &mut BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        WorthQueryDerivedViewRuntime,
    >,
    dependency_index: &WorthQueryComputedDependencyIndex,
    candidate_live_view_targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
    retained_live_rows: &BTreeMap<
        WorthQueryLiveArtifactTarget,
        Vec<crate::memory_workspace::WorthQueryEntity>,
    >,
    receipt: &WorthQueryMutationReceipt,
    mutation_metadata: &crate::runtime::WorthQueryMutationMetadata,
) -> Result<WorthQueryComputedRouteResult, IdentityDerivationDenial> {
    let mut affected = Vec::new();
    let mut refresh_fallback = false;
    let mut considered_view_count = 0;
    let mut candidates = dependency_index.live_candidates(candidate_live_view_targets);
    let mut emitted_by_view: BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        Vec<WorthQueryDerivedPatch>,
    > = BTreeMap::new();
    let refresh = WorthQueryRetainedRefreshContext::from_mutation(
        receipt.commit_identity.clone(),
        receipt.snapshot_identity.clone(),
        receipt
            .deltas
            .iter()
            .flat_map(|delta| delta.admitted_touched_aspects().iter().cloned()),
        mutation_metadata.clone(),
    );

    for view_target in topological_derived_order(derived_views) {
        if !candidates.contains(&view_target) {
            continue;
        }
        let Some(view) = derived_views.get(&view_target) else {
            continue;
        };
        considered_view_count += 1;
        let source_deltas = relevant_source_deltas(view, receipt, &emitted_by_view)?;
        if source_deltas.is_empty() {
            continue;
        }

        if !view.declaration.incremental() {
            let upstreams =
                retained_upstream_inputs(&view.declaration, retained_live_rows, derived_views);
            let Some(view) = derived_views.get_mut(&view_target) else {
                continue;
            };
            affected.push(view_target.clone());
            let mut patch = if let Some(maintainer) = view.maintainer.as_mut() {
                maintainer
                    .refresh_from_upstreams(
                        &view.declaration,
                        &refresh,
                        &upstreams,
                        &mut view.materialization,
                    )
                    .unwrap_or_else(|| {
                        WorthQueryDerivedPatch::whole_refresh_fallback(
                            view.declaration.name(),
                            receipt.commit_identity.clone(),
                            "derived view declared whole-refresh fallback",
                        )
                    })
            } else {
                WorthQueryDerivedPatch::whole_refresh_fallback(
                    view.declaration.name(),
                    receipt.commit_identity.clone(),
                    "derived view declared whole-refresh fallback",
                )
            };
            patch.bind_commit_identity(receipt.commit_identity.clone());
            if patch.is_refresh_fallback() {
                refresh_fallback = true;
            }
            emitted_by_view
                .entry(view_target.clone())
                .or_default()
                .push(patch.clone());
            view.patches.push(patch);
            candidates.extend(dependency_index.dependents(&view_target));
            continue;
        }

        let Some(view) = derived_views.get_mut(&view_target) else {
            continue;
        };
        for delta in source_deltas {
            affected.push(view_target.clone());
            let mut patch = if let Some(maintainer) = view.maintainer.as_mut() {
                maintainer.maintain(&view.declaration, &delta, &mut view.materialization)
            } else {
                WorthQueryDerivedPatch::incremental(
                    view.declaration.name(),
                    receipt.commit_identity.clone(),
                    delta.entity_identity.clone(),
                    effective_patch_aspects(&view.declaration, &delta),
                    WorthQueryDerivedPatchPayload::empty(),
                )
            };
            patch.bind_commit_identity(receipt.commit_identity.clone());
            if patch.is_refresh_fallback() {
                refresh_fallback = true;
            }
            emitted_by_view
                .entry(view_target.clone())
                .or_default()
                .push(patch.clone());
            view.patches.push(patch);
        }
        if emitted_by_view.contains_key(&view_target) {
            candidates.extend(dependency_index.dependents(&view_target));
        }
    }

    affected.sort();
    affected.dedup();
    Ok(WorthQueryComputedRouteResult::new(
        affected,
        refresh_fallback,
        considered_view_count,
    ))
}

pub(in crate::runtime) fn retained_live_view_names_for_candidates(
    derived_views: &BTreeMap<WorthQueryDerivedMaterializationTarget, WorthQueryDerivedViewRuntime>,
    dependency_index: &WorthQueryComputedDependencyIndex,
    candidate_live_view_targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
) -> BTreeSet<WorthQueryLiveArtifactTarget> {
    let candidate_live_view_targets = candidate_live_view_targets.into_iter().collect::<Vec<_>>();
    let mut retained = candidate_live_view_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut pending = dependency_index
        .live_candidates(candidate_live_view_targets)
        .into_iter()
        .collect::<Vec<_>>();
    let mut seen_views = BTreeSet::new();

    while let Some(view_target) = pending.pop() {
        if !seen_views.insert(view_target.clone()) {
            continue;
        }
        let Some(view) = derived_views.get(&view_target) else {
            continue;
        };
        retained.extend(
            view.declaration
                .upstream_live_views()
                .iter()
                .cloned()
                .map(WorthQueryLiveArtifactTarget::from_view_name),
        );
        pending.extend(dependency_index.dependents(&view_target));
    }
    retained
}
