pub(super) fn prepare_successor(
    session: &crate::facade::WorthUiActiveApplicationSession,
    application: &super::WorthUiPreparedApplicationActivation,
    successor: &mut crate::mounting::UiMountedGraphReplacementSuccessor,
    bindings: &crate::runtime::portal::UiPortalOverlayBindingLifecycle,
) -> Result<(), super::WorthUiApplicationCutoverDenial> {
    successor
        .prepare_occurrence_geometry_succession(
            &session.application,
            application.candidate_replacement_authority(),
            application.candidate_plan(),
            bindings,
        )
        .map_err(super::WorthUiApplicationCutoverDenial::OccurrenceGeometry)
}

pub(super) fn complete_candidate_surface(
    application: &super::WorthUiPreparedApplicationActivation,
    successor: &mut crate::mounting::UiMountedGraphReplacementSuccessor,
    lifecycle: &mut super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
    batch: crate::mounting::UiMountedSurfaceGeometryBatch,
) -> Result<(), super::WorthUiApplicationCutoverDenial> {
    let (bindings, mut scroll) = lifecycle.geometry_validation_inputs();
    let authority = crate::facade::entry::mounted_occurrence_geometry::UiMountedOccurrenceGeometryValidationAuthority::candidate(
        application,
        successor,
        bindings,
    );
    let validated = authority
        .validate(batch, scroll.as_deref_mut())
        .map_err(super::WorthUiApplicationCutoverDenial::OccurrenceGeometry)?;
    let (batch, _) = validated.into_parts();
    successor
        .replace_candidate_occurrence_geometry(batch, scroll.as_deref_mut())
        .map(|_| ())
        .map_err(super::WorthUiApplicationCutoverDenial::OccurrenceGeometry)
}
