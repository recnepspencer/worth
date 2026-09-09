use super::{owner_state, UiActiveOverlaySurfacePreparation};

pub(super) fn lower_surface(
    snapshot: &crate::runtime::overlay_composition::UiOverlayStackSnapshot,
    surface: &UiActiveOverlaySurfacePreparation,
    presentation: &crate::runtime::presentation_state::UiApplicationPresentationState,
    capabilities: &crate::capability::CapabilitySnapshot,
    appearance: Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    previous: Option<&owner_state::BackdropProjections>,
    work: owner_state::UiActiveBackdropAppearanceWork,
) -> Result<
    (
        crate::mounting::UiMountedAppearanceSurfaceOverlayInput,
        owner_state::UiActiveBackdropAppearanceCandidate,
    ),
    (),
> {
    let mut candidate = owner_state::UiActiveBackdropAppearanceCandidate {
        projections: Default::default(),
        work,
        sources: None,
    };
    for row in snapshot
        .participants()
        .iter()
        .filter_map(|participant| match participant {
            crate::runtime::overlay_composition::UiOverlayStackParticipant::Backdrop(row) => {
                Some(row)
            }
            crate::runtime::overlay_composition::UiOverlayStackParticipant::Portal(_) => None,
        })
    {
        let owner_snapshot = appearance.ok_or(())?;
        let vector = crate::runtime::appearance::UiBackdropAppearanceStateVector::seal(
            owner_snapshot,
            snapshot.runtime_surface(),
        );
        let declaration = surface.backdrops.get(&row.declaration()).ok_or(())?;
        let role = capabilities.appearance_roles().get(row.role()).ok_or(())?;
        let theme = presentation
            .appearance_theme_resolution_view(
                capabilities,
                role,
                snapshot.runtime_surface(),
                owner_snapshot.generation(),
            )
            .map_err(|_| ())?;
        candidate.work.candidates_visited += 1;
        let retained = previous
            .and_then(|projections| projections.get(&row.identity()))
            .and_then(|projection| {
                projection.reuse_for_overlay(declaration, role, &vector, &theme, snapshot)
            });
        let projection = match retained {
            Some(projection) => projection,
            None => {
                candidate.work.roles_resolved += 1;
                crate::runtime::appearance::UiAppearanceResolver::new()
                    .resolve_backdrop(row.identity(), declaration, role, &vector, &theme, snapshot)
                    .map_err(|_| ())?
            }
        };
        candidate.projections.insert(row.identity(), projection);
    }
    let output = crate::mounting::UiMountedAppearanceSurfaceOverlayInput::from_runtime_projections(
        snapshot,
        candidate.projections.values().cloned(),
    )
    .map_err(|_| ())?;
    Ok((output, candidate))
}
