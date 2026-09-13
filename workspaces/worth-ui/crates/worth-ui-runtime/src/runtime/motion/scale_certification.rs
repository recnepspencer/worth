/// Motion scale evidence for `RS-10`.
///
/// `inactive_tracks_sampled` is measured on the **same populated sampler** after
/// its tracks complete, so it can only stay zero if inactive retention truly
/// performs no per-frame work.
pub(crate) struct UiMotionScaleEvidence {
    pub(crate) active_tracks: u64,
    pub(crate) tracks_sampled: u64,
    pub(crate) inactive_tracks_sampled: u64,
    pub(crate) retained_inactive_tracks: u64,
    pub(crate) completed_terminals: u64,
    pub(crate) terminal_resources_zero: bool,
}

pub(crate) fn motion_scale_evidence() -> UiMotionScaleEvidence {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let semantic_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let presentation = presentation(frame);
    let mut state = super::UiMotionRuntimeState::new(
        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
    );
    let mut sampler =
        crate::mounting::presentation::motion_sampling::UiMountedMotionSampler::default();

    for index in 0..64_u64 {
        let target = super::UiMotionTargetIdentity::from_family_owner(
            semantic_surface,
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
            index + 1,
        );
        let request = super::UiMotionTransitionRequest::from_family_transition(
            target,
            1,
            2,
            presentation,
            geometry([0.0, 0.0, 40.0, 20.0]),
            true,
            presentation,
            geometry([4.0, 8.0, 44.0, 24.0]),
            true,
            super::UiMotionDeclaration::portal_entrance(),
        )
        .expect("motion scale request preserves one presentation binding");
        let proposal =
            crate::runtime::session::service_proposal::UiServiceProposalIdentity::for_test(
                index + 1,
            );
        let staged = state
            .stage(proposal, request)
            .expect("sixty-four tracks fit");
        let derived = state.derive(staged, frame);
        let committed = match state.commit_published(
            derived,
            crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
                proposal,
                7,
                crate::runtime::session::service_proposal::UiServiceProposalPublicationDisposition::Accepted,
            ),
            frame,
            presentation,
        ) {
            Ok(receipt) => receipt,
            Err((_, denial)) => panic!("motion scale track must commit: {denial:?}"),
        };
        sampler
            .install(committed)
            .expect("presentation sampler admits sixty-four active tracks");
    }

    let prepared = sampler
        .prepare_tick(1, presentation)
        .expect("active motion tick prepares");
    let sampled = sampler.commit_prepared(prepared);
    let tracks_considered = sampled.cost().tracks_considered();

    // Drive every track past its declared duration so the same populated sampler
    // still retains all sixty-four tracks while none of them remains active. A
    // later tick must then consider zero: inactive retention costs no per-frame
    // work. Measuring an empty sampler would prove nothing about retention.
    let completing = sampler
        .prepare_tick(
            u64::from(super::UiMotionDeclaration::portal_entrance().duration_ticks()) + 2,
            presentation,
        )
        .expect("completing motion tick prepares");
    let completed = sampler.commit_prepared(completing);
    let completed_terminals = completed.terminals().len() as u64;
    let inactive_tick = sampler
        .prepare_tick(
            u64::from(super::UiMotionDeclaration::portal_entrance().duration_ticks()) + 3,
            presentation,
        )
        .expect("retained inactive tick is valid");
    let inactive_work = sampler
        .commit_prepared(inactive_tick)
        .cost()
        .tracks_considered();
    let retained_inactive_tracks = sampler.certification_observation().1 as u64;
    let active_tracks = state.census().active_tracks() as u64;
    let owner_shutdown = state.shutdown().final_census().is_zero();
    let sampler_released = sampler.shutdown() == 64;
    UiMotionScaleEvidence {
        active_tracks,
        tracks_sampled: tracks_considered,
        inactive_tracks_sampled: inactive_work,
        retained_inactive_tracks,
        completed_terminals,
        terminal_resources_zero: owner_shutdown && sampler_released,
    }
}

fn geometry(components: [f32; 4]) -> Option<super::UiMotionSemanticGeometry> {
    Some(
        super::UiMotionSemanticGeometry::from_committed_components(
            components,
            worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        )
        .unwrap(),
    )
}

fn presentation(
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
) -> worth_ui_host_contract::UiHostObservationPresentationBasis {
    worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        frame,
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    )
}
