use super::candidate_preparation::CandidateOrigin;
use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedEffectFamily, UiSurfaceBindingGeneration,
};

use super::super::consumption_view::UiMountedHostPresentationAuthority;
use super::super::work_producer::{
    SuccessorIssueRequest, UiMountedPresentationCandidates, UiMountedPresentationState,
    UiMountedPresentationWorkProductionDenial,
};

pub(super) struct UiPreparedSurfacePresentation {
    pub(super) work: super::super::UiMountedPresentationWork,
    pub(super) expected_effects: Box<[UiMountedEffectFamily]>,
}

pub(super) struct UiPreparedFramePresentation {
    pub(super) surfaces: Vec<UiPreparedSurfacePresentation>,
    pub(super) candidates: UiMountedPresentationCandidates,
}

pub(super) fn issue(
    prepared: super::candidate_preparation::UiPreparedFrameCandidates,
    frame: &crate::mounting::UiPreparedMountedFrame,
    retained: &UiMountedPresentationCandidates,
    reconstruction_bindings: &std::collections::BTreeSet<UiSurfaceBindingGeneration>,
    authority: &UiMountedHostPresentationAuthority<'_>,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
) -> Result<UiPreparedFramePresentation, UiHostSurfacePresentationDenial> {
    let source = frame.presentation_delta_source();
    let mut surfaces = Vec::with_capacity(prepared.surfaces.len());
    let mut candidates = UiMountedPresentationCandidates::new();
    for (surface, prepared) in frame.surfaces().iter().zip(prepared.surfaces) {
        let binding = surface.requirement().binding();
        let predecessor = retained.get(&binding);
        if predecessor.map(UiMountedPresentationState::frame) != prepared.predecessor
            || reconstruction_bindings.contains(&binding) != prepared.reconstruction_required
        {
            return Err(UiHostSurfacePresentationDenial::StalePredecessor);
        }
        let candidate = prepared.state;
        let mut work = match prepared.origin {
            CandidateOrigin::Initial => {
                candidate.issue_initial(authority.presentation(), surface.projection())
            }
            CandidateOrigin::Successor => predecessor
                .expect("successor preparation requires an admitted predecessor")
                .issue_successor(
                    SuccessorIssueRequest::new(
                        &candidate,
                        source.changed_instances(),
                        source.frame().presentation_command_changes(),
                        authority.presentation(),
                    )
                    .with_surface_changed(
                        source.surface_changed(surface.requirement().semantic_surface()),
                    )
                    .with_source_predecessor(source.predecessor()),
                )
                .map_err(classify_work_error)?,
            CandidateOrigin::Reconstruction {
                predecessor,
                projection,
            } => candidate.issue_reconstruction(authority.presentation(), &projection, predecessor),
        };
        work.bind_layout_owner(surface.projection_owner());
        let appearance_sample_overrides =
            candidate.appearance_motion_overrides(frame.appearance_changed_instances());
        work.bind_appearance(
            frame.appearance_projection(),
            presentation,
            surface.requirement(),
            appearance_sample_overrides,
        )
        .map_err(|_| UiHostSurfacePresentationDenial::MalformedProjection)?;
        let expected_effects = candidate
            .expected_completion_effects(
                predecessor,
                &work,
                surface.requirement().presentation_mode(),
            )
            .into_boxed_slice();
        candidates.insert(binding, candidate);
        surfaces.push(UiPreparedSurfacePresentation {
            work,
            expected_effects,
        });
    }
    Ok(UiPreparedFramePresentation {
        surfaces,
        candidates,
    })
}
fn classify_work_error(
    denial: UiMountedPresentationWorkProductionDenial,
) -> UiHostSurfacePresentationDenial {
    match denial {
        UiMountedPresentationWorkProductionDenial::StalePredecessor => {
            UiHostSurfacePresentationDenial::StalePredecessor
        }
        UiMountedPresentationWorkProductionDenial::SurfaceChanged
        | UiMountedPresentationWorkProductionDenial::BindingChanged
        | UiMountedPresentationWorkProductionDenial::BaselineChanged => {
            UiHostSurfacePresentationDenial::SurfaceBindingChanged
        }
    }
}
