#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedLayoutSuccession {
    Exact,
    NestedRegionRetirement,
}

impl super::WorthUiApplicationSessionState {
    pub(crate) fn prepare_retained_layout_succession(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        candidate: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Result<
        crate::mounting::UiMountedOccurrenceGeometryState,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        mounted.prepare_retained_geometry_succession(
            self,
            candidate,
            self.runtime.active.active_plan_ref(),
            bindings,
        )
    }

    pub(crate) fn retains_mounted_layout(
        &self,
        candidate: &crate::runtime::WorthUiActiveExecutionPlan,
        candidate_graph: crate::graph::UiGraphAuthority<'_>,
        node: crate::graph::UiGraphNodeIdentity,
    ) -> Option<UiMountedLayoutSuccession> {
        use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;

        let previous = self.runtime.active.active_plan_ref();
        let root = |plan: &crate::runtime::WorthUiActiveExecutionPlan,
                    graph: crate::graph::UiGraphAuthority<'_>| {
            let digest = graph
                .lookup()
                .graph_node(node)?
                .value()
                .authored_provenance_digest();
            let index = plan.mounted_projection_plan_index(digest).ok()??;
            plan.mounted_projection_ordinary_meaning_with_identity(index)
                .map(|(identity, _)| identity)
        };
        let previous_root = root(previous, self.app.graph())?;
        let successor_root = candidate
            .mounted_projection_ordinary_meaning_for_identity(&previous_root)
            .map(|_| previous_root.clone())
            .or_else(|| root(candidate, candidate_graph))?;
        let mut pending = vec![(previous_root.clone(), successor_root, 0_usize)];
        let mut visited = std::collections::BTreeSet::new();
        let mut minimum_region_depth = None;
        let mut discovery = vec![(previous_root, 0_usize)];
        let mut discovered = std::collections::BTreeSet::new();
        while let Some((identity, depth)) = discovery.pop() {
            if !discovered.insert(identity.clone()) {
                continue;
            }
            let (_, meaning) =
                previous.mounted_projection_ordinary_meaning_for_identity(&identity)?;
            if matches!(meaning.as_ref(), Meaning::Layout(layout) if layout.region_descriptor().is_some())
            {
                minimum_region_depth =
                    Some(minimum_region_depth.map_or(depth, |current: usize| current.min(depth)));
            }
            discovery.extend(
                meaning
                    .dependency_identities()
                    .into_iter()
                    .map(|dependency| (dependency.to_owned(), depth.saturating_add(1))),
            );
        }
        let mut retired_nested_region = false;
        while let Some((previous_identity, successor_identity, depth)) = pending.pop() {
            if !visited.insert((previous_identity.clone(), successor_identity.clone())) {
                continue;
            }
            let Some((_, old)) =
                previous.mounted_projection_ordinary_meaning_for_identity(&previous_identity)
            else {
                return None;
            };
            let Some((_, next)) =
                candidate.mounted_projection_ordinary_meaning_for_identity(&successor_identity)
            else {
                return None;
            };
            if !old.same_mounted_layout_meaning(&next) {
                return None;
            }
            let dependencies = match (old.as_ref(), next.as_ref()) {
                (Meaning::ChildRange(old), Meaning::ChildRange(next)) => {
                    let mut pairs = Vec::new();
                    let mut old_index = 0;
                    let mut next_index = 0;
                    while old_index < old.child_identities().len()
                        && next_index < next.child_identities().len()
                    {
                        let old_identity = &old.child_identities()[old_index];
                        let next_identity = &next.child_identities()[next_index];
                        let (_, old_meaning) = previous
                            .mounted_projection_ordinary_meaning_for_identity(old_identity)?;
                        let (_, next_meaning) = candidate
                            .mounted_projection_ordinary_meaning_for_identity(next_identity)?;
                        if old_meaning.same_mounted_layout_meaning(&next_meaning) {
                            pairs.push((old_identity.as_str(), next_identity.as_str()));
                            old_index += 1;
                            next_index += 1;
                        } else if removable_nested_region(
                            old_meaning.as_ref(),
                            depth.saturating_add(1),
                            minimum_region_depth,
                        ) {
                            retired_nested_region = true;
                            old_index += 1;
                        } else {
                            return None;
                        }
                    }
                    if next_index != next.child_identities().len() {
                        return None;
                    }
                    while old_index < old.child_identities().len() {
                        let removed = &old.child_identities()[old_index];
                        let (_, meaning) =
                            previous.mounted_projection_ordinary_meaning_for_identity(removed)?;
                        if !removable_nested_region(
                            meaning.as_ref(),
                            depth.saturating_add(1),
                            minimum_region_depth,
                        ) {
                            return None;
                        }
                        retired_nested_region = true;
                        old_index += 1;
                    }
                    pairs
                }
                _ => {
                    let old = old.dependency_identities();
                    let next = next.dependency_identities();
                    if next.is_empty()
                        && !old.is_empty()
                        && old.iter().all(|identity| {
                            removable_nested_region_child_range(
                                previous,
                                identity,
                                depth.saturating_add(1),
                                minimum_region_depth,
                            )
                        })
                    {
                        retired_nested_region = true;
                        Vec::new()
                    } else if old.len() == next.len() {
                        old.into_iter().zip(next).collect()
                    } else {
                        return None;
                    }
                }
            };
            pending.extend(
                dependencies
                    .into_iter()
                    .map(|(old, next)| (old.to_owned(), next.to_owned(), depth.saturating_add(1))),
            );
        }
        Some(if retired_nested_region {
            UiMountedLayoutSuccession::NestedRegionRetirement
        } else {
            UiMountedLayoutSuccession::Exact
        })
    }
}

fn removable_nested_region(
    meaning: &crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning,
    depth: usize,
    minimum_region_depth: Option<usize>,
) -> bool {
    use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;
    matches!(
        meaning,
        Meaning::Layout(layout)
            if layout.region_descriptor().is_some()
                && minimum_region_depth.is_some_and(|minimum| depth > minimum)
    )
}

fn removable_nested_region_child_range(
    plan: &crate::runtime::WorthUiActiveExecutionPlan,
    identity: &str,
    depth: usize,
    minimum_region_depth: Option<usize>,
) -> bool {
    use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;
    let Some((_, meaning)) = plan.mounted_projection_ordinary_meaning_for_identity(identity) else {
        return false;
    };
    let Meaning::ChildRange(range) = meaning.as_ref() else {
        return false;
    };
    !range.child_identities().is_empty()
        && range.child_identities().iter().all(|child| {
            plan.mounted_projection_ordinary_meaning_for_identity(child)
                .is_some_and(|(_, meaning)| {
                    removable_nested_region(
                        meaning.as_ref(),
                        depth.saturating_add(1),
                        minimum_region_depth,
                    )
                })
        })
}
