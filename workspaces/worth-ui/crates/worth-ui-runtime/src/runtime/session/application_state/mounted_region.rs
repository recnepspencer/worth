use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedRegionDeclarationBinding {
    executed_region: String,
    region_kind: Box<str>,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    plan_index: u32,
    depth: usize,
    clipping: crate::capability::MosaicClippingPosture,
}

impl UiMountedRegionDeclarationBinding {
    pub(crate) fn executed_region(&self) -> &str {
        &self.executed_region
    }

    pub(crate) fn region_kind(&self) -> &str {
        &self.region_kind
    }

    pub(crate) fn declaration(&self) -> worth_ui_dsl::UiMosaicRegionDeclarationIdentity {
        self.declaration
    }

    pub(crate) fn clipping(&self) -> &crate::capability::MosaicClippingPosture {
        &self.clipping
    }

    pub(crate) const fn depth(&self) -> usize {
        self.depth
    }
}

impl super::WorthUiApplicationSessionState {
    pub(crate) fn mounted_region_declaration_for_plan_index(
        &self,
        surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
        plan_index: u32,
    ) -> Option<worth_ui_dsl::UiMosaicRegionDeclarationIdentity> {
        use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;
        let plan = self.runtime.active.active_plan_ref();
        let meaning = plan.mounted_projection_ordinary_meaning(plan_index)?;
        let Meaning::Layout(layout) = meaning.as_ref() else {
            return None;
        };
        let descriptor = layout.region_descriptor()?;
        self.authored_overlay_material()
            .overlay_declaration_bindings()
            .region_on_surface(surface, descriptor.id().as_str())
    }

    /// Enumerate only the exact owner's executed layout subtree at layout
    /// completion. Paint invalidation consumes the resulting carried bindings.
    pub(crate) fn mounted_region_declarations(
        &self,
        surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
        node: crate::graph::UiGraphNodeIdentity,
    ) -> (Vec<UiMountedRegionDeclarationBinding>, usize) {
        use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;
        let mut regions = Vec::new();
        let Some(graph_node) = self.app.graph().lookup().graph_node(node) else {
            return (regions, 0);
        };
        let plan = self.runtime.active.active_plan_ref();
        let Ok(Some(index)) =
            plan.mounted_projection_plan_index(graph_node.value().authored_provenance_digest())
        else {
            return (regions, 0);
        };
        let Some((root_identity, root)) =
            plan.mounted_projection_ordinary_meaning_with_identity(index)
        else {
            return (regions, 0);
        };
        let mut pending = vec![(root_identity, 0_usize, Some((index, root)))];
        let mut visited = BTreeSet::new();
        while let Some((identity, depth, preloaded)) = pending.pop() {
            if !visited.insert(identity.clone()) {
                continue;
            }
            let (plan_index, meaning) = match preloaded {
                Some(preloaded) => preloaded,
                None => {
                    let Some(meaning) =
                        plan.mounted_projection_ordinary_meaning_for_identity(&identity)
                    else {
                        continue;
                    };
                    meaning
                }
            };
            if let Meaning::Layout(layout) = meaning.as_ref() {
                if let Some(descriptor) = layout.region_descriptor() {
                    if let (Some(declaration), Some(clipping)) = (
                        self.authored_overlay_material()
                            .overlay_declaration_bindings()
                            .region_on_surface(surface, descriptor.id().as_str()),
                        descriptor.clipping().cloned(),
                    ) {
                        regions.push(UiMountedRegionDeclarationBinding {
                            executed_region: identity.clone(),
                            region_kind: descriptor.id().as_str().into(),
                            declaration,
                            plan_index,
                            depth,
                            clipping,
                        });
                    }
                }
            }
            pending.extend(
                meaning
                    .dependency_identities()
                    .into_iter()
                    .map(|identity| (identity.to_owned(), depth.saturating_add(1), None)),
            );
        }
        regions.sort_by_key(|binding| binding.plan_index);
        (regions, visited.len())
    }
}
