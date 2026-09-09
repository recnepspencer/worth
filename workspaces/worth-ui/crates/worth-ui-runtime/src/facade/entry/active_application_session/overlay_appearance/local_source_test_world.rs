use super::*;

pub(super) struct SourceWorld {
    app: crate::facade::entry::WorthUiHostNeutralApp,
    pub(super) portals: UiPortalRuntimeState,
    pub(super) motion: UiMotionRuntimeState,
    pub(super) surface: UiActiveOverlaySurfacePreparation,
    pub(super) foreign: UiActiveOverlaySurfacePreparation,
    pub(super) declarations: Vec<UiPortalDeclarationId>,
    pub(super) portal_ids: Vec<UiPortalIdentity>,
    targets: Vec<UiMotionTargetIdentity>,
    presentation: UiHostObservationPresentationBasis,
    sequence: u64,
}

impl SourceWorld {
    pub(super) fn new() -> Self {
        let app = crate::facade::WorthUi::app()
            .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
            .freeze()
            .unwrap();
        let generation = app.generation_identity().clone();
        let surfaces = [
            UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        ];
        let declarations = (1..=512)
            .map(|index| UiPortalDeclarationId::new(index).unwrap())
            .collect::<Vec<_>>();
        let declared = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
        let presentation = UiHostObservationPresentationBasis::new(
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiMountedFrameIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            UiHostPresentationEpoch::issued_by_host(1),
        );
        let mut portals =
            UiPortalRuntimeState::new(crate::runtime::UiServiceStatePersistencePosture::Ephemeral);
        let mut bindings = UiPortalOverlayBindingOwner::new(generation.clone(), surfaces[0]);
        let mut portal_ids = Vec::new();
        let geometry =
            crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation);
        let viewport = crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
            geometry.clip_bounds(),
            presentation,
        );
        for index in 0..513 {
            let portal = UiPortalIdentity::for_owner(UiPortalOwnerIdentity::from_mounted_owner(
                crate::graph::UiGraphNodeIdentity::new(index as u64 + 1),
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
            ));
            let surface = if index == 512 {
                surfaces[1]
            } else {
                surfaces[0]
            };
            let request = UiPortalServiceRequest::open(
                portal,
                idempotency(index as u64 + 1),
                geometry,
                Some(viewport),
                surface,
            );
            let transition = portals.prepare(request).unwrap();
            portals.commit_published(transition).unwrap();
            if index < 512 {
                bindings.bind(declarations[index], portal).unwrap();
            }
            portal_ids.push(portal);
        }
        let mut foreign_bindings = UiPortalOverlayBindingOwner::new(generation, surfaces[1]);
        foreign_bindings
            .bind(declarations[0], portal_ids[512])
            .unwrap();
        let backdrop = backdrop(declared, declarations[0]);
        let surface = UiActiveOverlaySurfacePreparation {
            declaration_surface: declared,
            runtime_surface: surfaces[0],
            portal_stack: None,
            staged_motion: None,
            extent: None,
            bindings,
            backdrops: BTreeMap::from([(backdrop.identity(), backdrop)]),
        };
        let foreign = UiActiveOverlaySurfacePreparation {
            declaration_surface: UiSemanticSurfaceDeclarationIdentity::new(2).unwrap(),
            runtime_surface: surfaces[1],
            portal_stack: None,
            staged_motion: None,
            extent: None,
            bindings: foreign_bindings,
            backdrops: BTreeMap::new(),
        };
        let targets = portal_ids
            .iter()
            .take(32)
            .map(|portal| {
                UiMotionTargetIdentity::from_family_owner(
                    surfaces[0],
                    portal.owner().mounted_instance_identity(),
                    portal.diagnostic_value(),
                )
            })
            .collect::<Vec<_>>();
        let mut world = Self {
            app,
            portals,
            motion: UiMotionRuntimeState::new(
                crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
            ),
            surface,
            foreign,
            declarations,
            portal_ids,
            targets,
            presentation,
            sequence: 1000,
        };
        for index in 0..32 {
            world.retarget(index, 1, 2);
        }
        world
    }

    pub(super) fn close(&mut self, index: usize) {
        self.sequence += 1;
        let portal = self.portal_ids[index];
        let transition = self
            .portals
            .prepare(UiPortalServiceRequest::close(
                portal,
                idempotency(self.sequence),
                UiPortalDismissalCause::ExplicitOwnerRequest,
                self.surface.runtime_surface,
            ))
            .unwrap();
        self.portals.commit_published(transition).unwrap();
        self.surface.bindings.remove(portal);
    }

    pub(super) fn current(&self) -> UiOverlayStackSnapshot {
        let generation = self.app.generation_identity();
        let presentation =
            crate::runtime::presentation_state::UiApplicationPresentationState::activate(
                self.app.capabilities(),
            )
            .overlay_owner_export(
                generation.clone(),
                UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            );
        let extent = crate::runtime::UiMountedOverlayExtentOwner::new(
            generation.clone(),
            self.surface.declaration_surface,
            self.surface.runtime_surface,
            1,
            crate::runtime::UiCommittedOverlayExtentBounds::new(0., 0., 800., 600.).unwrap(),
            [],
        )
        .unwrap();
        let (observed, bindings) = capture(
            Some(&self.portals),
            Some(&self.motion),
            &self.surface,
            None,
            &mut Default::default(),
        );
        let (portals, _) =
            localized_portal_source(&observed, None, None, &bindings, &mut Default::default())
                .unwrap();
        let (motion, _) = localized_motion_source(
            &observed,
            None,
            None,
            &bindings,
            self.surface.runtime_surface,
            &mut Default::default(),
        )
        .unwrap();
        let sources = crate::runtime::overlay_composition::UiOverlayOwnerSources::from_exports(
            generation,
            portals,
            &extent,
            &presentation,
            &self.surface.bindings,
            motion,
        );
        crate::runtime::overlay_composition::UiOverlayCompositionOwnerLifecycle::admit_from_owners(
            self.surface.backdrops.values().cloned(),
            1,
            sources,
        )
        .unwrap()
        .current()
        .unwrap()
        .clone()
    }

    pub(super) fn retarget(&mut self, index: usize, before: u64, after: u64) {
        self.sequence += 1;
        let request = UiMotionTransitionRequest::from_family_transition(
            self.targets[index],
            before,
            after,
            self.presentation,
            None,
            false,
            self.presentation,
            None,
            true,
            UiMotionDeclaration::portal_entrance(),
        )
        .unwrap();
        self.motion.commit_declared_transition_for_test(
            self.sequence,
            request,
            self.presentation.frame(),
            self.presentation,
        );
    }
}

fn idempotency(
    sequence: u64,
) -> crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity {
    crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(1, sequence)
}

fn backdrop(
    surface: UiSemanticSurfaceDeclarationIdentity,
    portal: UiPortalDeclarationId,
) -> UiBackdropDeclaration {
    let mut role = UiAppearanceRole::authoring(
        UiAppearanceRoleIdentity::new("test.local-source.scrim").unwrap(),
    )
    .applies_to_backdrop();
    for aspect in [UiAppearanceAspect::Background, UiAppearanceAspect::Opacity] {
        role = role
            .cover(
                aspect,
                UiAppearancePartitionAuthoring::new([]).with_cell(
                    UiAppearanceCell::when([]).uses_slot(
                        UiThemeSlotIdentity::new(format!("test.local-source.{aspect:?}")).unwrap(),
                        aspect.value_kind(),
                    ),
                ),
            )
            .unwrap();
    }
    UiBackdropDeclaration::admit(
        UiBackdropIdentity::new(1).unwrap(),
        surface,
        UiBackdropScope::PerPortalInstance(portal),
        UiBackdropExtentBasis::SurfaceViewport(surface),
        UiBackdropPresenceBasis::WhilePortalPresented(portal),
        UiBackdropMotionBasis::PortalPresentation(portal),
        UiBackdropPlacement::ImmediatelyBeforePortal(portal),
        &role.build().unwrap(),
    )
    .unwrap()
}
