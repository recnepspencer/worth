use std::collections::{BTreeMap, BTreeSet};

mod target_index;

use super::{WorthQueryLiveArtifactTarget, WorthQueryRuntime};
pub(crate) use target_index::{
    WorthQueryInstalledTargetSelection, WorthQueryInstalledTargetSelectionWork,
};

struct WorthQueryInstalledLiveRoute {
    impact_classifier: crate::domain_installation::WorthQueryInstalledLiveImpactClassifier,
}

pub(super) enum WorthQueryInstalledLiveMutationClassification {
    Ordinary,
    InstalledUnaffected,
    Affected(crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact),
}

impl WorthQueryInstalledLiveMutationClassification {
    pub(super) const fn is_installed_but_unaffected(&self) -> bool {
        matches!(self, Self::InstalledUnaffected)
    }

    pub(super) fn into_impact(
        self,
    ) -> Option<crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact> {
        match self {
            Self::Affected(impact) => Some(impact),
            Self::Ordinary | Self::InstalledUnaffected => None,
        }
    }
}

#[derive(Default)]
pub(super) struct WorthQueryInstalledLiveRoutes {
    routes: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryInstalledLiveRoute>,
    target_index: target_index::WorthQueryInstalledLiveTargetIndex,
}

impl WorthQueryInstalledLiveRoutes {
    pub(super) fn contains_target(&self, target: &WorthQueryLiveArtifactTarget) -> bool {
        self.routes.contains_key(target)
    }

    pub(super) fn affected_targets(
        &self,
        mutation: &crate::memory_workspace::WorthQueryMutationDelta,
    ) -> WorthQueryInstalledTargetSelection {
        self.target_index.affected_targets(mutation)
    }

    pub(super) fn classify_live_mutation(
        &self,
        target: &WorthQueryLiveArtifactTarget,
        mutation: &crate::memory_workspace::WorthQueryMutationDelta,
        affected_installed_targets: &BTreeSet<WorthQueryLiveArtifactTarget>,
    ) -> WorthQueryInstalledLiveMutationClassification {
        let Some(route) = self.routes.get(target) else {
            return WorthQueryInstalledLiveMutationClassification::Ordinary;
        };
        if !affected_installed_targets.contains(target) {
            return WorthQueryInstalledLiveMutationClassification::InstalledUnaffected;
        }
        WorthQueryInstalledLiveMutationClassification::Affected(
            route.impact_classifier.classify(mutation),
        )
    }
}

impl WorthQueryRuntime {
    pub(crate) fn register_installed_live_route(
        &mut self,
        target: WorthQueryLiveArtifactTarget,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    ) {
        self.unregister_installed_live_route(&target);
        let impact_classifier =
            crate::domain_installation::WorthQueryInstalledLiveImpactClassifier::from_closure(
                closure,
            );
        let target_collection = self
            .live_subscriptions
            .get(&target)
            .expect("installed live route retains its live subscription")
            .request
            .target_collection_identity()
            .as_str()
            .to_owned();
        self.installed_live_routes.target_index.register(
            target.clone(),
            target_collection,
            impact_classifier.routing_selector(),
        );
        self.installed_live_routes
            .routes
            .insert(target, WorthQueryInstalledLiveRoute { impact_classifier });
    }

    pub(crate) fn unregister_installed_live_route(
        &mut self,
        target: &WorthQueryLiveArtifactTarget,
    ) {
        if self.installed_live_routes.routes.remove(target).is_none() {
            return;
        }
        self.installed_live_routes.target_index.unregister(target);
    }
}
