pub(super) fn prepare_frame(
    session: &crate::facade::WorthUiActiveApplicationSession,
    application: &super::WorthUiPreparedApplicationActivation,
    mounted: &crate::mounting::UiMountedGraphReplacementSuccessor,
    semantic_content: crate::mounting::UiMountedSemanticContentInput,
    request: crate::mounting::UiMountedFrameRequest,
) -> Result<
    (
        crate::mounting::UiPreparedMountedFrame,
        super::super::owner_succession::UiPreparedApplicationOwnerSuccession,
    ),
    super::WorthUiApplicationCutoverDenial,
> {
    let authority = application.candidate_replacement_authority();
    let capability_report = session.host_session.capability_report();
    let text = session
        .presentation
        .prepare_text_succession(
            session.capabilities(),
            authority.capabilities(),
            crate::graph::UiGraphAuthority::new(&application.candidate_graph),
        )
        .map_err(super::WorthUiApplicationCutoverDenial::MountedFrame)?;
    let frame = super::super::mounted_frame::prepare_candidate_mounted_frame(
        application,
        mounted,
        crate::graph::UiGraphAuthority::new(&application.candidate_graph),
        super::super::mounted_frame::UiMountedReplacementFrameBasis {
            generation: authority.generation_identity().clone(),
            host_session: session.host_session.identity().as_u64(),
            protocol: session.host_session.protocol(),
            capability_generation: capability_report.observation_generation(),
            capability_profile_digest: capability_report.profile_identity_digest(),
        },
        semantic_content,
        text.project()
            .map_err(super::WorthUiApplicationCutoverDenial::MountedFrame)?,
        request,
    )
    .map_err(super::WorthUiApplicationCutoverDenial::MountedFrame)?;
    let owners = super::super::owner_succession::UiPreparedApplicationOwnerSuccession::prepare(
        session,
        application,
        mounted,
        &frame,
        text,
    )?;
    let themes = application
        .appearance_succession
        .as_ref()
        .expect("prepared replacement retains appearance succession")
        .theme();
    let frame = frame.resolve_appearance(crate::facade::entry::appearance_projection::UiAppearanceFrameProjection {
        application_session_identity: session.identity,
        generation_identity: authority.generation_identity().clone(),
        graph: crate::graph::UiGraphAuthority::new(&application.candidate_graph),
        capabilities: authority.capabilities(),
        consumed_facts: authority.consumed_fact_index(),
        intent_catalog: authority.intent_catalog(),
        mounted: &session.mounted,
        presentation: &session.presentation,
        appearance_owner_snapshot: owners.snapshot(),
        phase:
            crate::facade::entry::appearance_projection::UiAppearanceProjectionPhase::Replacement {
                mounted,
                themes,
            },
    })
    .map_err(super::WorthUiApplicationCutoverDenial::MountedFrame)?;
    Ok((frame, owners))
}
