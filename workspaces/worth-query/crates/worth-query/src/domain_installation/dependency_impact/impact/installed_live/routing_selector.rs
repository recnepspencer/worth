use std::collections::BTreeSet;

use worth_foundational::facade::{AspectKey, CanonicalFieldPath};

use super::WorthQueryInstalledLiveImpactClassifier;

#[derive(Clone)]
pub(crate) struct WorthQueryInstalledLiveRoutingSelector {
    pub(crate) aspect_routes: BTreeSet<AspectKey>,
    pub(crate) whole_aspect_routes: BTreeSet<AspectKey>,
    pub(crate) field_routes: BTreeSet<(AspectKey, CanonicalFieldPath)>,
    pub(crate) structural_creation: bool,
    pub(crate) broad: bool,
    pub(crate) empty_touch: bool,
}

impl WorthQueryInstalledLiveImpactClassifier {
    pub(crate) fn routing_selector(&self) -> WorthQueryInstalledLiveRoutingSelector {
        WorthQueryInstalledLiveRoutingSelector {
            aspect_routes: self.aspect_roles.keys().cloned().collect(),
            whole_aspect_routes: self
                .whole_roles
                .keys()
                .chain(self.ambiguous_native_aspects.iter())
                .chain(self.conditional_aspects.iter())
                .cloned()
                .collect(),
            field_routes: self.field_routes.clone(),
            structural_creation: !self.structural_roles.is_empty(),
            broad: self.conditional_broad_locality,
            empty_touch: !self.conditional_aspects.is_empty(),
        }
    }
}
