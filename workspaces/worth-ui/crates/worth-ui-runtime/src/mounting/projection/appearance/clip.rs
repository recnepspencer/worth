use worth_ui_host_contract::{UiAppearanceClip, UiAppearanceVisualBounds};

/// Ancestor coverage is independent of a mechanic's allocation and radius.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceClip {
    Unclipped,
    Suppressed,
    Ancestor(UiAppearanceClip),
    Unresolved(UiMountedAppearanceClipDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceClipDenial {
    #[cfg(test)]
    ExecutedPlanUnavailable,
    MountedGeometryUnavailable,
    Geometry(super::UiMountedAppearanceGeometryDenial),
    #[cfg(test)]
    MosaicBindingUnavailable(crate::graph::UiGraphNodeIdentity),
    PortalBindingUnavailable(crate::graph::UiGraphNodeIdentity),
    ScrollBindingUnavailable(crate::graph::UiGraphNodeIdentity),
}

impl UiMountedAppearanceClip {
    pub(super) fn require_resolved(self) -> Result<(), super::UiMountedAppearanceLoweringDenial> {
        match self {
            Self::Unresolved(denial) => Err(
                super::UiMountedAppearanceLoweringDenial::AncestorClip(denial),
            ),
            Self::Unclipped | Self::Ancestor(_) | Self::Suppressed => Ok(()),
        }
    }

    pub(super) fn for_visual_bounds(
        self,
        bounds: UiAppearanceVisualBounds,
    ) -> Result<UiAppearanceClip, super::UiMountedAppearanceLoweringDenial> {
        self.require_resolved()?;
        match self {
            Self::Ancestor(clip) => Ok(clip),
            // The finite host transport needs no coverage beyond this mechanic.
            // This encodes absence of extra clipping, not an ancestor boundary.
            Self::Unclipped => {
                UiAppearanceClip::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
                    .map_err(|_| {
                        super::UiMountedAppearanceLoweringDenial::NodeAllocationUnavailable
                    })
            }
            Self::Unresolved(_) => unreachable!("clip resolution was required above"),
            Self::Suppressed => unreachable!("suppressed nodes have no visual mechanics"),
        }
    }
}

/// Inspect authored topology only while constructing a mounted node. Ordinary
/// owner-state updates consume the carried result without walking ancestors.
pub(crate) fn derive_unbound_ancestry(
    graph: crate::graph::UiGraphAuthority<'_>,
    _plan: crate::mounting::UiMountedPlanProjectionSource<'_>,
    target: crate::graph::UiGraphNodeIdentity,
    portal_child: bool,
    mosaic_clips: &[worth_ui_host_contract::UiMountedCanonicalBox],
    scroll_clips: Result<
        &[worth_ui_host_contract::UiMountedCanonicalBox],
        crate::graph::UiGraphNodeIdentity,
    >,
) -> Result<(UiMountedAppearanceClip, usize), crate::mounting::UiMountedProjectionDenial> {
    use crate::declaration::UiDeclarationPlanningOperatorKind as Operator;
    use UiMountedAppearanceClipDenial as Denial;
    let mut portal_requirement = portal_child.then_some(target);
    let mut cursor = Some(target);
    let mut entries = 1usize;
    while let Some(node) = cursor {
        let topology = graph
            .lookup()
            .topology_node(node)
            .ok_or(crate::mounting::UiMountedProjectionDenial::UnknownGraphNode)?;
        let record = graph
            .lookup()
            .graph_node(node)
            .ok_or(crate::mounting::UiMountedProjectionDenial::UnknownGraphNode)?;
        entries = entries
            .checked_add(2)
            .ok_or(crate::mounting::UiMountedProjectionDenial::CostCounterOverflow)?;
        if record.value().operator_kind() == Operator::PortalAnchor {
            portal_requirement.get_or_insert(node);
        }
        cursor = topology.value().parent_node_identity();
    }
    let scroll_clips = match scroll_clips {
        Ok(clips) => clips,
        Err(node) => {
            return Ok((
                UiMountedAppearanceClip::Unresolved(Denial::ScrollBindingUnavailable(node)),
                entries,
            ))
        }
    };
    let clip = match intersect_ancestor_clips(mosaic_clips.iter().chain(scroll_clips).copied()) {
        Ok(clip) => clip,
        Err(clip) => return Ok((clip, entries)),
    };
    Ok((
        clip.map_or_else(
            || {
                portal_requirement.map_or(UiMountedAppearanceClip::Unclipped, |node| {
                    UiMountedAppearanceClip::Unresolved(Denial::PortalBindingUnavailable(node))
                })
            },
            UiMountedAppearanceClip::Ancestor,
        ),
        entries,
    ))
}

/// The coverage ancestor clips leave what they enclose: suppressed when one
/// has no area or together they share none. Scroll moves clips, so clip
/// derivation, a Scroll pose, and hit rows read this one rule.
pub(crate) fn ancestor_clip(
    clips: impl IntoIterator<Item = worth_ui_host_contract::UiMountedCanonicalBox>,
) -> UiMountedAppearanceClip {
    match intersect_ancestor_clips(clips) {
        Ok(Some(clip)) => UiMountedAppearanceClip::Ancestor(clip),
        Ok(None) => UiMountedAppearanceClip::Unclipped,
        Err(clip) => clip,
    }
}

/// The coverage ancestor clips share, in order, or `None` when there are
/// none. A clip with no area or no shared coverage suppresses, and geometry
/// that cannot be read leaves the clip unresolved.
fn intersect_ancestor_clips(
    clips: impl IntoIterator<Item = worth_ui_host_contract::UiMountedCanonicalBox>,
) -> Result<Option<UiAppearanceClip>, UiMountedAppearanceClip> {
    use super::UiMountedAppearanceGeometryDenial as Geometry;
    let mut clip = None;
    for bounds in clips {
        let bounds = match super::geometry::allocation(bounds) {
            Ok(bounds) => bounds,
            Err(Geometry::AllocationHasNoArea | Geometry::EmptyAtCanonicalPrecision) => {
                return Err(UiMountedAppearanceClip::Suppressed)
            }
            Err(denial) => {
                return Err(UiMountedAppearanceClip::Unresolved(
                    UiMountedAppearanceClipDenial::Geometry(denial),
                ))
            }
        };
        let bounds = UiAppearanceClip::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
            .expect("canonical ancestor geometry has area");
        clip = match clip {
            None => Some(bounds),
            Some(current) => {
                Some(intersect_clips(current, bounds).ok_or(UiMountedAppearanceClip::Suppressed)?)
            }
        };
    }
    Ok(clip)
}

pub(super) fn intersect_clips(
    a: UiAppearanceClip,
    b: UiAppearanceClip,
) -> Option<UiAppearanceClip> {
    let x = a.x().max(b.x());
    let y = a.y().max(b.y());
    let right =
        (i64::from(a.x()) + i64::from(a.width())).min(i64::from(b.x()) + i64::from(b.width()));
    let bottom =
        (i64::from(a.y()) + i64::from(a.height())).min(i64::from(b.y()) + i64::from(b.height()));
    let width = u32::try_from(right - i64::from(x)).ok()?;
    let height = u32::try_from(bottom - i64::from(y)).ok()?;
    UiAppearanceClip::new(x, y, width, height).ok()
}
