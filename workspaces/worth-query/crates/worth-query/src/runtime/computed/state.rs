use super::*;
use crate::runtime::{WorthQueryDerivedMaterializationTarget, WorthQueryLiveArtifactTarget};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum WorthQueryComputedAdmissionError {
    MissingUpstreamLive { upstream: String },
    MissingUpstreamComputed { upstream: String },
    SelfDependency,
    Cycle { upstream: String },
}

impl WorthQueryComputedAdmissionError {
    pub(in crate::runtime) fn message(&self) -> String {
        match self {
            Self::MissingUpstreamLive { upstream } => {
                format!("live upstream '{upstream}' is not declared")
            }
            Self::MissingUpstreamComputed { upstream } => {
                format!("computed upstream '{upstream}' is not declared")
            }
            Self::SelfDependency => "computed declaration may not depend on itself".to_string(),
            Self::Cycle { upstream } => {
                format!("computed declaration would create a cycle through '{upstream}'")
            }
        }
    }
}

pub(in crate::runtime) struct WorthQueryDerivedViewRuntime {
    pub(in crate::runtime) declaration: WorthQueryDerivedView,
    pub(in crate::runtime) patches: Vec<WorthQueryDerivedPatch>,
    pub(in crate::runtime) materialization: WorthQueryDerivedViewMaterialization,
    pub(in crate::runtime) maintainer: Option<Box<dyn WorthQueryDerivedViewMaintainer>>,
}

impl WorthQueryDerivedViewRuntime {
    pub(in crate::runtime) fn new(
        declaration: WorthQueryDerivedView,
        maintainer: Option<Box<dyn WorthQueryDerivedViewMaintainer>>,
    ) -> Self {
        Self {
            declaration,
            patches: Vec::new(),
            materialization: WorthQueryDerivedViewMaterialization::default(),
            maintainer,
        }
    }
}

#[derive(Default)]
pub(in crate::runtime) struct WorthQueryComputedDependencyIndex {
    live_to_computed:
        BTreeMap<WorthQueryLiveArtifactTarget, BTreeSet<WorthQueryDerivedMaterializationTarget>>,
    computed_to_dependents: BTreeMap<
        WorthQueryDerivedMaterializationTarget,
        BTreeSet<WorthQueryDerivedMaterializationTarget>,
    >,
    unscoped_authoritative_computed: BTreeSet<WorthQueryDerivedMaterializationTarget>,
}

impl WorthQueryComputedDependencyIndex {
    pub(in crate::runtime) fn register(&mut self, declaration: &WorthQueryDerivedView) {
        self.unregister(declaration.name());
        let target = WorthQueryDerivedMaterializationTarget::new(declaration.name());
        if declaration.upstream_live_views().is_empty()
            && declaration.upstream_derived_views().is_empty()
        {
            self.unscoped_authoritative_computed.insert(target.clone());
        }
        for live_view in declaration.upstream_live_views() {
            self.live_to_computed
                .entry(WorthQueryLiveArtifactTarget::from_view_name(
                    live_view.clone(),
                ))
                .or_default()
                .insert(target.clone());
        }
        for upstream_computed in declaration.upstream_derived_views() {
            self.computed_to_dependents
                .entry(WorthQueryDerivedMaterializationTarget::new(
                    upstream_computed.clone(),
                ))
                .or_default()
                .insert(target.clone());
        }
    }

    fn unregister(&mut self, view_name: &str) {
        let target = WorthQueryDerivedMaterializationTarget::new(view_name);
        self.unscoped_authoritative_computed.remove(&target);
        remove_from_index(&mut self.live_to_computed, view_name);
        remove_from_index(&mut self.computed_to_dependents, view_name);
    }

    pub(in crate::runtime) fn live_candidates(
        &self,
        live_view_targets: impl IntoIterator<Item = WorthQueryLiveArtifactTarget>,
    ) -> BTreeSet<WorthQueryDerivedMaterializationTarget> {
        let mut candidates = self.unscoped_authoritative_computed.clone();
        for live_view_target in live_view_targets {
            if let Some(computed_views) = self.live_to_computed.get(&live_view_target) {
                candidates.extend(computed_views.iter().cloned());
            }
        }
        candidates
    }

    pub(in crate::runtime) fn has_live_dependents(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> bool {
        self.live_to_computed
            .get(target)
            .is_some_and(|dependents| !dependents.is_empty())
    }

    pub(in crate::runtime) fn dependents(
        &self,
        computed_target: &WorthQueryDerivedMaterializationTarget,
    ) -> impl Iterator<Item = WorthQueryDerivedMaterializationTarget> + '_ + use<'_> {
        self.computed_to_dependents
            .get(computed_target)
            .into_iter()
            .flatten()
            .cloned()
    }
}

fn remove_from_index<T: Ord + Clone>(
    index: &mut BTreeMap<T, BTreeSet<WorthQueryDerivedMaterializationTarget>>,
    view_name: &str,
) {
    let target = WorthQueryDerivedMaterializationTarget::new(view_name);
    let empty_keys: Vec<T> = index
        .iter_mut()
        .filter_map(|(key, values)| {
            values.remove(&target);
            values.is_empty().then(|| key.clone())
        })
        .collect();
    for key in empty_keys {
        index.remove(&key);
    }
}
